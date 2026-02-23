import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition, applyUrlDefaults } from './../../wayfinder'
/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
*/
export const index = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

index.definition = {
    methods: ["get","head"],
    url: '/wallets',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
*/
index.url = (options?: RouteQueryOptions) => {
    return index.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
*/
index.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
*/
index.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: index.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
*/
const indexForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
*/
indexForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:19
* @route '/wallets'
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
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:31
* @route '/wallets'
*/
export const store = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

store.definition = {
    methods: ["post"],
    url: '/wallets',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:31
* @route '/wallets'
*/
store.url = (options?: RouteQueryOptions) => {
    return store.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:31
* @route '/wallets'
*/
store.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:31
* @route '/wallets'
*/
const storeForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:31
* @route '/wallets'
*/
storeForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: store.url(options),
    method: 'post',
})

store.form = storeForm

/**
* @see \App\Http\Controllers\WalletController::destroy
* @see app/Http/Controllers/WalletController.php:45
* @route '/wallets/{wallet}'
*/
export const destroy = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

destroy.definition = {
    methods: ["delete"],
    url: '/wallets/{wallet}',
} satisfies RouteDefinition<["delete"]>

/**
* @see \App\Http\Controllers\WalletController::destroy
* @see app/Http/Controllers/WalletController.php:45
* @route '/wallets/{wallet}'
*/
destroy.url = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
    if (typeof args === 'string' || typeof args === 'number') {
        args = { wallet: args }
    }

    if (typeof args === 'object' && !Array.isArray(args) && 'id' in args) {
        args = { wallet: args.id }
    }

    if (Array.isArray(args)) {
        args = {
            wallet: args[0],
        }
    }

    args = applyUrlDefaults(args)

    const parsedArgs = {
        wallet: typeof args.wallet === 'object'
        ? args.wallet.id
        : args.wallet,
    }

    return destroy.definition.url
            .replace('{wallet}', parsedArgs.wallet.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::destroy
* @see app/Http/Controllers/WalletController.php:45
* @route '/wallets/{wallet}'
*/
destroy.delete = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

/**
* @see \App\Http\Controllers\WalletController::destroy
* @see app/Http/Controllers/WalletController.php:45
* @route '/wallets/{wallet}'
*/
const destroyForm = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: destroy.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'DELETE',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::destroy
* @see app/Http/Controllers/WalletController.php:45
* @route '/wallets/{wallet}'
*/
destroyForm.delete = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
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
* @see \App\Http\Controllers\WalletController::assignStrategy
* @see app/Http/Controllers/WalletController.php:60
* @route '/wallets/{wallet}/strategies'
*/
export const assignStrategy = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: assignStrategy.url(args, options),
    method: 'post',
})

assignStrategy.definition = {
    methods: ["post"],
    url: '/wallets/{wallet}/strategies',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\WalletController::assignStrategy
* @see app/Http/Controllers/WalletController.php:60
* @route '/wallets/{wallet}/strategies'
*/
assignStrategy.url = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
    if (typeof args === 'string' || typeof args === 'number') {
        args = { wallet: args }
    }

    if (typeof args === 'object' && !Array.isArray(args) && 'id' in args) {
        args = { wallet: args.id }
    }

    if (Array.isArray(args)) {
        args = {
            wallet: args[0],
        }
    }

    args = applyUrlDefaults(args)

    const parsedArgs = {
        wallet: typeof args.wallet === 'object'
        ? args.wallet.id
        : args.wallet,
    }

    return assignStrategy.definition.url
            .replace('{wallet}', parsedArgs.wallet.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::assignStrategy
* @see app/Http/Controllers/WalletController.php:60
* @route '/wallets/{wallet}/strategies'
*/
assignStrategy.post = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: assignStrategy.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::assignStrategy
* @see app/Http/Controllers/WalletController.php:60
* @route '/wallets/{wallet}/strategies'
*/
const assignStrategyForm = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: assignStrategy.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::assignStrategy
* @see app/Http/Controllers/WalletController.php:60
* @route '/wallets/{wallet}/strategies'
*/
assignStrategyForm.post = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: assignStrategy.url(args, options),
    method: 'post',
})

assignStrategy.form = assignStrategyForm

/**
* @see \App\Http\Controllers\WalletController::removeStrategy
* @see app/Http/Controllers/WalletController.php:78
* @route '/wallets/{wallet}/strategies/{strategy}'
*/
export const removeStrategy = (args: { wallet: number | { id: number }, strategy: number | { id: number } } | [wallet: number | { id: number }, strategy: number | { id: number } ], options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: removeStrategy.url(args, options),
    method: 'delete',
})

removeStrategy.definition = {
    methods: ["delete"],
    url: '/wallets/{wallet}/strategies/{strategy}',
} satisfies RouteDefinition<["delete"]>

/**
* @see \App\Http\Controllers\WalletController::removeStrategy
* @see app/Http/Controllers/WalletController.php:78
* @route '/wallets/{wallet}/strategies/{strategy}'
*/
removeStrategy.url = (args: { wallet: number | { id: number }, strategy: number | { id: number } } | [wallet: number | { id: number }, strategy: number | { id: number } ], options?: RouteQueryOptions) => {
    if (Array.isArray(args)) {
        args = {
            wallet: args[0],
            strategy: args[1],
        }
    }

    args = applyUrlDefaults(args)

    const parsedArgs = {
        wallet: typeof args.wallet === 'object'
        ? args.wallet.id
        : args.wallet,
        strategy: typeof args.strategy === 'object'
        ? args.strategy.id
        : args.strategy,
    }

    return removeStrategy.definition.url
            .replace('{wallet}', parsedArgs.wallet.toString())
            .replace('{strategy}', parsedArgs.strategy.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::removeStrategy
* @see app/Http/Controllers/WalletController.php:78
* @route '/wallets/{wallet}/strategies/{strategy}'
*/
removeStrategy.delete = (args: { wallet: number | { id: number }, strategy: number | { id: number } } | [wallet: number | { id: number }, strategy: number | { id: number } ], options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: removeStrategy.url(args, options),
    method: 'delete',
})

/**
* @see \App\Http\Controllers\WalletController::removeStrategy
* @see app/Http/Controllers/WalletController.php:78
* @route '/wallets/{wallet}/strategies/{strategy}'
*/
const removeStrategyForm = (args: { wallet: number | { id: number }, strategy: number | { id: number } } | [wallet: number | { id: number }, strategy: number | { id: number } ], options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: removeStrategy.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'DELETE',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::removeStrategy
* @see app/Http/Controllers/WalletController.php:78
* @route '/wallets/{wallet}/strategies/{strategy}'
*/
removeStrategyForm.delete = (args: { wallet: number | { id: number }, strategy: number | { id: number } } | [wallet: number | { id: number }, strategy: number | { id: number } ], options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: removeStrategy.url(args, {
        [options?.mergeQuery ? 'mergeQuery' : 'query']: {
            _method: 'DELETE',
            ...(options?.query ?? options?.mergeQuery ?? {}),
        }
    }),
    method: 'post',
})

removeStrategy.form = removeStrategyForm

const wallets = {
    index: Object.assign(index, index),
    store: Object.assign(store, store),
    destroy: Object.assign(destroy, destroy),
    assignStrategy: Object.assign(assignStrategy, assignStrategy),
    removeStrategy: Object.assign(removeStrategy, removeStrategy),
}

export default wallets