import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition, applyUrlDefaults } from './../../wayfinder'
/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:37
* @route '/strategies/generate'
*/
export const generate = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: generate.url(options),
    method: 'post',
})

generate.definition = {
    methods: ["post"],
    url: '/strategies/generate',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:37
* @route '/strategies/generate'
*/
generate.url = (options?: RouteQueryOptions) => {
    return generate.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:37
* @route '/strategies/generate'
*/
generate.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: generate.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:37
* @route '/strategies/generate'
*/
const generateForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: generate.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:37
* @route '/strategies/generate'
*/
generateForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: generate.url(options),
    method: 'post',
})

generate.form = generateForm

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:22
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
* @see app/Http/Controllers/StrategyController.php:22
* @route '/strategies'
*/
index.url = (options?: RouteQueryOptions) => {
    return index.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:22
* @route '/strategies'
*/
index.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:22
* @route '/strategies'
*/
index.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: index.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:22
* @route '/strategies'
*/
const indexForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:22
* @route '/strategies'
*/
indexForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::index
* @see app/Http/Controllers/StrategyController.php:22
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
* @see app/Http/Controllers/StrategyController.php:32
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
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies/create'
*/
create.url = (options?: RouteQueryOptions) => {
    return create.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies/create'
*/
create.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies/create'
*/
create.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: create.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies/create'
*/
const createForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:32
* @route '/strategies/create'
*/
createForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:32
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
* @see app/Http/Controllers/StrategyController.php:55
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
* @see app/Http/Controllers/StrategyController.php:55
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
* @see app/Http/Controllers/StrategyController.php:55
* @route '/strategies/{strategy}'
*/
show.get = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:55
* @route '/strategies/{strategy}'
*/
show.head = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: show.url(args, options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:55
* @route '/strategies/{strategy}'
*/
const showForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:55
* @route '/strategies/{strategy}'
*/
showForm.get = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:55
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
* @see app/Http/Controllers/StrategyController.php:91
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
* @see app/Http/Controllers/StrategyController.php:91
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
* @see app/Http/Controllers/StrategyController.php:91
* @route '/strategies/{strategy}'
*/
update.put = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'put'> => ({
    url: update.url(args, options),
    method: 'put',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:91
* @route '/strategies/{strategy}'
*/
update.patch = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'patch'> => ({
    url: update.url(args, options),
    method: 'patch',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:91
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
* @see app/Http/Controllers/StrategyController.php:91
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
* @see app/Http/Controllers/StrategyController.php:91
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
* @see app/Http/Controllers/StrategyController.php:98
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
* @see app/Http/Controllers/StrategyController.php:98
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
* @see app/Http/Controllers/StrategyController.php:98
* @route '/strategies/{strategy}'
*/
destroy.delete = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:98
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
* @see app/Http/Controllers/StrategyController.php:98
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
* @see app/Http/Controllers/StrategyController.php:48
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
* @see app/Http/Controllers/StrategyController.php:48
* @route '/strategies'
*/
store.url = (options?: RouteQueryOptions) => {
    return store.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:48
* @route '/strategies'
*/
store.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:48
* @route '/strategies'
*/
const storeForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:48
* @route '/strategies'
*/
storeForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: store.url(options),
    method: 'post',
})

store.form = storeForm

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:113
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
* @see app/Http/Controllers/StrategyController.php:113
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
* @see app/Http/Controllers/StrategyController.php:113
* @route '/strategies/{strategy}/activate'
*/
activate.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: activate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:113
* @route '/strategies/{strategy}/activate'
*/
const activateForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: activate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:113
* @route '/strategies/{strategy}/activate'
*/
activateForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: activate.url(args, options),
    method: 'post',
})

activate.form = activateForm

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:126
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
* @see app/Http/Controllers/StrategyController.php:126
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
* @see app/Http/Controllers/StrategyController.php:126
* @route '/strategies/{strategy}/deactivate'
*/
deactivate.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: deactivate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:126
* @route '/strategies/{strategy}/deactivate'
*/
const deactivateForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: deactivate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:126
* @route '/strategies/{strategy}/deactivate'
*/
deactivateForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: deactivate.url(args, options),
    method: 'post',
})

deactivate.form = deactivateForm

/**
* @see \App\Http\Controllers\StrategyController::kill
* @see app/Http/Controllers/StrategyController.php:139
* @route '/strategies/{strategy}/kill'
*/
export const kill = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: kill.url(args, options),
    method: 'post',
})

kill.definition = {
    methods: ["post"],
    url: '/strategies/{strategy}/kill',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\StrategyController::kill
* @see app/Http/Controllers/StrategyController.php:139
* @route '/strategies/{strategy}/kill'
*/
kill.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return kill.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::kill
* @see app/Http/Controllers/StrategyController.php:139
* @route '/strategies/{strategy}/kill'
*/
kill.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: kill.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::kill
* @see app/Http/Controllers/StrategyController.php:139
* @route '/strategies/{strategy}/kill'
*/
const killForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: kill.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::kill
* @see app/Http/Controllers/StrategyController.php:139
* @route '/strategies/{strategy}/kill'
*/
killForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: kill.url(args, options),
    method: 'post',
})

kill.form = killForm

/**
* @see \App\Http\Controllers\StrategyController::unkill
* @see app/Http/Controllers/StrategyController.php:155
* @route '/strategies/{strategy}/unkill'
*/
export const unkill = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: unkill.url(args, options),
    method: 'post',
})

unkill.definition = {
    methods: ["post"],
    url: '/strategies/{strategy}/unkill',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\StrategyController::unkill
* @see app/Http/Controllers/StrategyController.php:155
* @route '/strategies/{strategy}/unkill'
*/
unkill.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return unkill.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::unkill
* @see app/Http/Controllers/StrategyController.php:155
* @route '/strategies/{strategy}/unkill'
*/
unkill.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: unkill.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::unkill
* @see app/Http/Controllers/StrategyController.php:155
* @route '/strategies/{strategy}/unkill'
*/
const unkillForm = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: unkill.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::unkill
* @see app/Http/Controllers/StrategyController.php:155
* @route '/strategies/{strategy}/unkill'
*/
unkillForm.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: unkill.url(args, options),
    method: 'post',
})

unkill.form = unkillForm

const strategies = {
    generate: Object.assign(generate, generate),
    index: Object.assign(index, index),
    create: Object.assign(create, create),
    show: Object.assign(show, show),
    update: Object.assign(update, update),
    destroy: Object.assign(destroy, destroy),
    store: Object.assign(store, store),
    activate: Object.assign(activate, activate),
    deactivate: Object.assign(deactivate, deactivate),
    kill: Object.assign(kill, kill),
    unkill: Object.assign(unkill, unkill),
}

export default strategies