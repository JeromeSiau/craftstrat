import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition, applyUrlDefaults } from './../../wayfinder'
/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
export const index = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

index.definition = {
    methods: ["get","head"],
    url: '/backtests',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
index.url = (options?: RouteQueryOptions) => {
    return index.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
index.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
index.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: index.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
const indexForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
indexForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BacktestController::index
* @see app/Http/Controllers/BacktestController.php:17
* @route '/backtests'
*/
indexForm.head = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url({
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'HEAD',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'get',
})

index.form = indexForm

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
export const show = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: show.url(args, options),
    method: 'get',
})

show.definition = {
    methods: ["get","head"],
    url: '/backtests/{result}',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
show.url = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
    if (typeof args === 'string' || typeof args === 'number') {
        args = { result: args }
    }

    if (typeof args === 'object' && !Array.isArray(args) && 'id' in args) {
        args = { result: args.id }
    }

    if (Array.isArray(args)) {
        args = {
            result: args[0],
        }
    }

    args = applyUrlDefaults(args)

    const parsedArgs = {
        result: typeof args.result === 'object'
        ? args.result.id
        : args.result,
    }

    return show.definition.url
            .replace('{result}', parsedArgs.result.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
show.get = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
show.head = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: show.url(args, options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
const showForm = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
showForm.get = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BacktestController::show
* @see app/Http/Controllers/BacktestController.php:27
* @route '/backtests/{result}'
*/
showForm.head = (args: { result: number | { id: number } } | [result: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'HEAD',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'get',
})

show.form = showForm

/**
* @see \App\Http\Controllers\BacktestController::run
* @see app/Http/Controllers/BacktestController.php:38
* @route '/strategies/{strategy}/backtest'
*/
export const run = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: run.url(args, options),
    method: 'post',
})

run.definition = {
    methods: ["post"],
    url: '/strategies/{strategy}/backtest',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\BacktestController::run
* @see app/Http/Controllers/BacktestController.php:38
* @route '/strategies/{strategy}/backtest'
*/
run.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
    if (typeof args === 'string' || typeof args === 'number') {
        args = { strategy: args }
    }

    if (typeof args === 'object' && !Array.isArray(args) && 'id' in args) {
        args = { strategy: args.id }
    }

    if (Array.isArray(args)) {
        args = {
            strategy: args[0],
        }
    }

    args = applyUrlDefaults(args)

    const parsedArgs = {
        strategy: typeof args.strategy === 'object'
        ? args.strategy.id
        : args.strategy,
    }

    return run.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\BacktestController::run
* @see app/Http/Controllers/BacktestController.php:38
* @route '/strategies/{strategy}/backtest'
*/
run.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: run.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\BacktestController::run
* @see app/Http/Controllers/BacktestController.php:38
* @route '/strategies/{strategy}/backtest'
*/
const runForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: run.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\BacktestController::run
* @see app/Http/Controllers/BacktestController.php:38
* @route '/strategies/{strategy}/backtest'
*/
runForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: run.url(args, options),
    method: 'post',
})

run.form = runForm

const backtests = {
    index: Object.assign(index, index),
    show: Object.assign(show, show),
    run: Object.assign(run, run),
}

export default backtests