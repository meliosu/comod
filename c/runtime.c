#include <stdbool.h>
#include <stdint.h>

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

// Task structure
typedef struct task {
    void (*function)(void);
    struct task *next;
} task_t;

// Thread pool structure
typedef struct {
    pthread_t *workers; // Array of worker threads
    int num_workers;    // Number of worker threads

    task_t *task_queue;      // Head of task queue
    task_t *task_queue_tail; // Tail of task queue for fast enqueue

    pthread_mutex_t mutex;        // Mutex for thread synchronization
    pthread_cond_t cond_has_task; // Condition variable for signaling new tasks
    pthread_cond_t
        cond_all_done; // Condition variable for signaling all tasks completed

    int active_tasks; // Number of tasks in queue or being processed
    bool shutdown;    // Flag for shutdown
} threadpool_t;

// Global thread pool
static threadpool_t *pool = NULL;

// Worker thread function
static void *worker(void *arg) {
    while (true) {
        pthread_mutex_lock(&pool->mutex);

        // Wait for tasks if none available
        while (pool->task_queue == NULL && !pool->shutdown) {
            pthread_cond_wait(&pool->cond_has_task, &pool->mutex);
        }

        // Exit if shutting down and no tasks
        if (pool->shutdown && pool->task_queue == NULL) {
            pthread_mutex_unlock(&pool->mutex);
            pthread_exit(NULL);
        }

        // Get a task
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
            // Execute the task
            task->function();
            free(task);

            // Update active task count
            pthread_mutex_lock(&pool->mutex);
            pool->active_tasks--;

            // Signal if all tasks are done
            if (pool->active_tasks == 0 && pool->task_queue == NULL) {
                pthread_cond_broadcast(&pool->cond_all_done);
            }
            pthread_mutex_unlock(&pool->mutex);
        }
    }

    return NULL;
}

// Initialize the thread pool with specified number of workers
int tp_init(int num_workers) {
    if (pool != NULL || num_workers <= 0) {
        return -1; // Already initialized or invalid parameter
    }

    // Allocate pool structure
    pool = (threadpool_t *)malloc(sizeof(threadpool_t));
    if (pool == NULL) {
        return -1;
    }

    // Initialize members
    pool->num_workers = num_workers;
    pool->task_queue = NULL;
    pool->task_queue_tail = NULL;
    pool->active_tasks = 0;
    pool->shutdown = false;

    // Initialize synchronization primitives
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

    // Allocate and create worker threads
    pool->workers = (pthread_t *)malloc(num_workers * sizeof(pthread_t));
    if (pool->workers == NULL) {
        pthread_cond_destroy(&pool->cond_all_done);
        pthread_cond_destroy(&pool->cond_has_task);
        pthread_mutex_destroy(&pool->mutex);
        free(pool);
        return -1;
    }

    // Create worker threads
    for (int i = 0; i < num_workers; i++) {
        if (pthread_create(&pool->workers[i], NULL, worker, NULL) != 0) {
            // Clean up on error
            pool->shutdown = true;
            pthread_cond_broadcast(&pool->cond_has_task);

            // Wait for any created threads to finish
            for (int j = 0; j < i; j++) {
                pthread_join(pool->workers[j], NULL);
            }

            // Free resources
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

// Add a task to the queue
int tp_enqueue(void (*task)(void)) {
    if (pool == NULL || task == NULL) {
        return -1;
    }

    // Create a new task
    task_t *new_task = (task_t *)malloc(sizeof(task_t));
    if (new_task == NULL) {
        return -1;
    }

    new_task->function = task;
    new_task->next = NULL;

    // Add task to queue
    pthread_mutex_lock(&pool->mutex);

    if (pool->task_queue == NULL) {
        pool->task_queue = new_task;
        pool->task_queue_tail = new_task;
    } else {
        pool->task_queue_tail->next = new_task;
        pool->task_queue_tail = new_task;
    }

    pool->active_tasks++;

    // Signal that a new task is available
    pthread_cond_signal(&pool->cond_has_task);

    pthread_mutex_unlock(&pool->mutex);

    return 0;
}

// Wait for all tasks to complete
int tp_wait(void) {
    if (pool == NULL) {
        return -1;
    }

    pthread_mutex_lock(&pool->mutex);

    // Wait until all tasks are done
    while (pool->active_tasks > 0) {
        pthread_cond_wait(&pool->cond_all_done, &pool->mutex);
    }

    pthread_mutex_unlock(&pool->mutex);

    return 0;
}

// Clean up the thread pool (optional helper function)
int tp_destroy(void) {
    if (pool == NULL) {
        return -1;
    }

    // Set shutdown flag and wake up all workers
    pthread_mutex_lock(&pool->mutex);
    pool->shutdown = true;
    pthread_cond_broadcast(&pool->cond_has_task);
    pthread_mutex_unlock(&pool->mutex);

    // Wait for workers to finish
    for (int i = 0; i < pool->num_workers; i++) {
        pthread_join(pool->workers[i], NULL);
    }

    // Free resources
    free(pool->workers);

    // Free any remaining tasks
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
