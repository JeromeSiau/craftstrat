import { queryParams, type RouteQueryOptions, type RouteDefinition, applyUrlDefaults } from './../../../../wayfinder'
/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:38
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
* @see app/Http/Controllers/StrategyController.php:38
* @route '/strategies/generate'
*/
generate.url = (options?: RouteQueryOptions) => {
    return generate.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::generate
* @see app/Http/Controllers/StrategyController.php:38
* @route '/strategies/generate'
*/
generate.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: generate.url(options),
    method: 'post',
})

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
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:33
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
* @see app/Http/Controllers/StrategyController.php:33
* @route '/strategies/create'
*/
create.url = (options?: RouteQueryOptions) => {
    return create.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:33
* @route '/strategies/create'
*/
create.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: create.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::create
* @see app/Http/Controllers/StrategyController.php:33
* @route '/strategies/create'
*/
create.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: create.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:65
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
* @see app/Http/Controllers/StrategyController.php:65
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
* @see app/Http/Controllers/StrategyController.php:65
* @route '/strategies/{strategy}'
*/
show.get = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: show.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::show
* @see app/Http/Controllers/StrategyController.php:65
* @route '/strategies/{strategy}'
*/
show.head = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: show.url(args, options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::edit
* @see app/Http/Controllers/StrategyController.php:56
* @route '/strategies/{strategy}/edit'
*/
export const edit = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: edit.url(args, options),
    method: 'get',
})

edit.definition = {
    methods: ["get","head"],
    url: '/strategies/{strategy}/edit',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\StrategyController::edit
* @see app/Http/Controllers/StrategyController.php:56
* @route '/strategies/{strategy}/edit'
*/
edit.url = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return edit.definition.url
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::edit
* @see app/Http/Controllers/StrategyController.php:56
* @route '/strategies/{strategy}/edit'
*/
edit.get = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: edit.url(args, options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\StrategyController::edit
* @see app/Http/Controllers/StrategyController.php:56
* @route '/strategies/{strategy}/edit'
*/
edit.head = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: edit.url(args, options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:101
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
* @see app/Http/Controllers/StrategyController.php:101
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
* @see app/Http/Controllers/StrategyController.php:101
* @route '/strategies/{strategy}'
*/
update.put = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'put'> => ({
    url: update.url(args, options),
    method: 'put',
})

/**
* @see \App\Http\Controllers\StrategyController::update
* @see app/Http/Controllers/StrategyController.php:101
* @route '/strategies/{strategy}'
*/
update.patch = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'patch'> => ({
    url: update.url(args, options),
    method: 'patch',
})

/**
* @see \App\Http\Controllers\StrategyController::destroy
* @see app/Http/Controllers/StrategyController.php:108
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
* @see app/Http/Controllers/StrategyController.php:108
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
* @see app/Http/Controllers/StrategyController.php:108
* @route '/strategies/{strategy}'
*/
destroy.delete = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:49
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
* @see app/Http/Controllers/StrategyController.php:49
* @route '/strategies'
*/
store.url = (options?: RouteQueryOptions) => {
    return store.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\StrategyController::store
* @see app/Http/Controllers/StrategyController.php:49
* @route '/strategies'
*/
store.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::activate
* @see app/Http/Controllers/StrategyController.php:123
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
* @see app/Http/Controllers/StrategyController.php:123
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
* @see app/Http/Controllers/StrategyController.php:123
* @route '/strategies/{strategy}/activate'
*/
activate.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: activate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::deactivate
* @see app/Http/Controllers/StrategyController.php:136
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
* @see app/Http/Controllers/StrategyController.php:136
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
* @see app/Http/Controllers/StrategyController.php:136
* @route '/strategies/{strategy}/deactivate'
*/
deactivate.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: deactivate.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::kill
* @see app/Http/Controllers/StrategyController.php:149
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
* @see app/Http/Controllers/StrategyController.php:149
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
* @see app/Http/Controllers/StrategyController.php:149
* @route '/strategies/{strategy}/kill'
*/
kill.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: kill.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\StrategyController::unkill
* @see app/Http/Controllers/StrategyController.php:165
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
* @see app/Http/Controllers/StrategyController.php:165
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
* @see app/Http/Controllers/StrategyController.php:165
* @route '/strategies/{strategy}/unkill'
*/
unkill.post = (args: { strategy: number | { id: number } } | [strategy: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: unkill.url(args, options),
    method: 'post',
})

const StrategyController = { generate, index, create, show, edit, update, destroy, store, activate, deactivate, kill, unkill }

export default StrategyController