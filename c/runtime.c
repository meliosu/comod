#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

typedef int8_t i8;
typedef uint8_t u8;
typedef int16_t i16;
typedef uint16_t u16;
typedef int32_t i32;
typedef uint32_t u32;
typedef int64_t i64;
typedef uint64_t u64;
typedef float f32;
typedef double f64;
typedef void *ptr;

#define WAITING 0
#define RUNNING 1
#define STOPPED 2

#include <pthread.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct task {
    void (*function)(void);
    struct task *next;
} task_t;

typedef struct {
    pthread_t *workers;
    int num_workers;

    task_t *task_queue;
    task_t *task_queue_tail;

    pthread_mutex_t mutex;
    pthread_cond_t cond_has_task;
    pthread_cond_t cond_all_done;

    int active_tasks;
    bool shutdown;
} threadpool_t;

static threadpool_t *pool = NULL;

static void *worker(void *arg) {
    while (true) {
        pthread_mutex_lock(&pool->mutex);

        while (pool->task_queue == NULL && !pool->shutdown) {
            pthread_cond_wait(&pool->cond_has_task, &pool->mutex);
        }

        if (pool->shutdown && pool->task_queue == NULL) {
            pthread_mutex_unlock(&pool->mutex);
            pthread_exit(NULL);
        }

        task_t *task = NULL;
        if (pool->task_queue != NULL) {
            task = pool->task_queue;
            pool->task_queue = task->next;

            if (pool->task_queue == NULL) {
                pool->task_queue_tail = NULL;
            }
        }

        pthread_mutex_unlock(&pool->mutex);

        if (task != NULL) {
            task->function();
            free(task);

            pthread_mutex_lock(&pool->mutex);
            pool->active_tasks--;

            if (pool->active_tasks == 0 && pool->task_queue == NULL) {
                pthread_cond_broadcast(&pool->cond_all_done);
            }

            pthread_mutex_unlock(&pool->mutex);
        }
    }

    return NULL;
}

int tp_init(int num_workers) {
    if (pool != NULL || num_workers <= 0) {
        return -1;
    }

    pool = (threadpool_t *)malloc(sizeof(threadpool_t));
    if (pool == NULL) {
        return -1;
    }

    pool->num_workers = num_workers;
    pool->task_queue = NULL;
    pool->task_queue_tail = NULL;
    pool->active_tasks = 0;
    pool->shutdown = false;

    if (pthread_mutex_init(&pool->mutex, NULL) != 0) {
        free(pool);
        return -1;
    }

    if (pthread_cond_init(&pool->cond_has_task, NULL) != 0) {
        pthread_mutex_destroy(&pool->mutex);
        free(pool);
        return -1;
    }

    if (pthread_cond_init(&pool->cond_all_done, NULL) != 0) {
        pthread_cond_destroy(&pool->cond_has_task);
        pthread_mutex_destroy(&pool->mutex);
        free(pool);
        return -1;
    }

    pool->workers = (pthread_t *)malloc(num_workers * sizeof(pthread_t));
    if (pool->workers == NULL) {
        pthread_cond_destroy(&pool->cond_all_done);
        pthread_cond_destroy(&pool->cond_has_task);
        pthread_mutex_destroy(&pool->mutex);
        free(pool);
        return -1;
    }

    for (int i = 0; i < num_workers; i++) {
        if (pthread_create(&pool->workers[i], NULL, worker, NULL) != 0) {
            pool->shutdown = true;
            pthread_cond_broadcast(&pool->cond_has_task);

            for (int j = 0; j < i; j++) {
                pthread_join(pool->workers[j], NULL);
            }

            free(pool->workers);
            pthread_cond_destroy(&pool->cond_all_done);
            pthread_cond_destroy(&pool->cond_has_task);
            pthread_mutex_destroy(&pool->mutex);
            free(pool);
            pool = NULL;

            return -1;
        }
    }

    return 0;
}

int tp_enqueue(void (*task)(void)) {
    if (pool == NULL || task == NULL) {
        return -1;
    }

    task_t *new_task = (task_t *)malloc(sizeof(task_t));
    if (new_task == NULL) {
        return -1;
    }

    new_task->function = task;
    new_task->next = NULL;

    pthread_mutex_lock(&pool->mutex);

    if (pool->task_queue == NULL) {
        pool->task_queue = new_task;
        pool->task_queue_tail = new_task;
    } else {
        pool->task_queue_tail->next = new_task;
        pool->task_queue_tail = new_task;
    }

    pool->active_tasks++;

    pthread_cond_signal(&pool->cond_has_task);

    pthread_mutex_unlock(&pool->mutex);

    return 0;
}

int tp_wait(void) {
    if (pool == NULL) {
        return -1;
    }

    pthread_mutex_lock(&pool->mutex);

    while (pool->active_tasks > 0) {
        pthread_cond_wait(&pool->cond_all_done, &pool->mutex);
    }

    pthread_mutex_unlock(&pool->mutex);

    return 0;
}

int tp_destroy(void) {
    if (pool == NULL) {
        return -1;
    }

    pthread_mutex_lock(&pool->mutex);
    pool->shutdown = true;
    pthread_cond_broadcast(&pool->cond_has_task);
    pthread_mutex_unlock(&pool->mutex);

    for (int i = 0; i < pool->num_workers; i++) {
        pthread_join(pool->workers[i], NULL);
    }

    free(pool->workers);

    task_t *task = pool->task_queue;
    while (task != NULL) {
        task_t *next = task->next;
        free(task);
        task = next;
    }

    pthread_mutex_destroy(&pool->mutex);
    pthread_cond_destroy(&pool->cond_has_task);
    pthread_cond_destroy(&pool->cond_all_done);

    free(pool);
    pool = NULL;

    return 0;
}
