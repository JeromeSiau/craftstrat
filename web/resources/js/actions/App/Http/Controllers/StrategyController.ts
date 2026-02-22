import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition, applyUrlDefaults } from './../../../../wayfinder'
/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
*/
export const index = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

index.definition = {
    methods: ["get","head"],
    url: '/strategies',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
*/
index.url = (options?: RouteQueryOptions) => {
    return index.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
*/
index.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
*/
index.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: index.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
*/
const indexForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
*/
indexForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:17
* @route '/strategies'
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
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
export const create = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: create.url(options),
    method: 'get',
})

create.definition = {
    methods: ["get","head"],
    url: '/strategies/create',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
create.url = (options?: RouteQueryOptions) => {
    return create.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
create.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
create.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: create.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
const createForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
createForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:27
* @route '/strategies/create'
*/
createForm.head = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: create.url({
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'HEAD',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'get',
})

create.form = createForm

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
export const show = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: show.url(args, options),
    method: 'get',
})

show.definition = {
    methods: ["get","head"],
    url: '/strategies/{strategy}',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
show.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return show.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
show.get = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
show.head = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: show.url(args, options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
const showForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
showForm.get = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:39
* @route '/strategies/{strategy}'
*/
showForm.head = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
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
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
export const update = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'put'> => ({
    url: update.url(args, options),
    method: 'put',
})

update.definition = {
    methods: ["put","patch"],
    url: '/strategies/{strategy}',
} satisfies RouteDefinition<["put","patch"]>

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
update.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return update.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
update.put = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'put'> => ({
    url: update.url(args, options),
    method: 'put',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
update.patch = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'patch'> => ({
    url: update.url(args, options),
    method: 'patch',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
const updateForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: update.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'PUT',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
updateForm.put = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: update.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'PUT',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:50
* @route '/strategies/{strategy}'
*/
updateForm.patch = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: update.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'PATCH',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

update.form = updateForm

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:57
* @route '/strategies/{strategy}'
*/
export const destroy = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

destroy.definition = {
    methods: ["delete"],
    url: '/strategies/{strategy}',
} satisfies RouteDefinition<["delete"]>

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:57
* @route '/strategies/{strategy}'
*/
destroy.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return destroy.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:57
* @route '/strategies/{strategy}'
*/
destroy.delete = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:57
* @route '/strategies/{strategy}'
*/
const destroyForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: destroy.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'DELETE',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:57
* @route '/strategies/{strategy}'
*/
destroyForm.delete = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: destroy.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'DELETE',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

destroy.form = destroyForm

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies'
*/
export const store = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

store.definition = {
    methods: ["post"],
    url: '/strategies',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies'
*/
store.url = (options?: RouteQueryOptions) => {
    return store.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies'
*/
store.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies'
*/
const storeForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies'
*/
storeForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: store.url(options),
    method: 'post',
})

store.form = storeForm

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:72
* @route '/strategies/{strategy}/activate'
*/
export const activate = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: activate.url(args, options),
    method: 'post',
})

activate.definition = {
    methods: ["post"],
    url: '/strategies/{strategy}/activate',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:72
* @route '/strategies/{strategy}/activate'
*/
activate.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return activate.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:72
* @route '/strategies/{strategy}/activate'
*/
activate.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: activate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:72
* @route '/strategies/{strategy}/activate'
*/
const activateForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: activate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:72
* @route '/strategies/{strategy}/activate'
*/
activateForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: activate.url(args, options),
    method: 'post',
})

activate.form = activateForm

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:85
* @route '/strategies/{strategy}/deactivate'
*/
export const deactivate = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: deactivate.url(args, options),
    method: 'post',
})

deactivate.definition = {
    methods: ["post"],
    url: '/strategies/{strategy}/deactivate',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:85
* @route '/strategies/{strategy}/deactivate'
*/
deactivate.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return deactivate.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:85
* @route '/strategies/{strategy}/deactivate'
*/
deactivate.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: deactivate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:85
* @route '/strategies/{strategy}/deactivate'
*/
const deactivateForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: deactivate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:85
* @route '/strategies/{strategy}/deactivate'
*/
deactivateForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: deactivate.url(args, options),
    method: 'post',
})

deactivate.form = deactivateForm

const StrategyController = { index, create, show, update, destroy, store, activate, deactivate }

export default StrategyController