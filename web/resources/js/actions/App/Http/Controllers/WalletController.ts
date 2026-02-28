import { queryParams, type RouteQueryOptions, type RouteDefinition, applyUrlDefaults } from './../../../../wayfinder'
/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:20
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
* @see app/Http/Controllers/WalletController.php:20
* @route '/wallets'
*/
index.url = (options?: RouteQueryOptions) => {
    return index.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:20
* @route '/wallets'
*/
index.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\WalletController::index
* @see app/Http/Controllers/WalletController.php:20
* @route '/wallets'
*/
index.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: index.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:32
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
* @see app/Http/Controllers/WalletController.php:32
* @route '/wallets'
*/
store.url = (options?: RouteQueryOptions) => {
    return store.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::store
* @see app/Http/Controllers/WalletController.php:32
* @route '/wallets'
*/
store.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: store.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::destroy
* @see app/Http/Controllers/WalletController.php:49
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
* @see app/Http/Controllers/WalletController.php:49
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
* @see app/Http/Controllers/WalletController.php:49
* @route '/wallets/{wallet}'
*/
destroy.delete = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: destroy.url(args, options),
    method: 'delete',
})

/**
* @see \App\Http\Controllers\WalletController::retryDeploy
* @see app/Http/Controllers/WalletController.php:68
* @route '/wallets/{wallet}/retry'
*/
export const retryDeploy = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: retryDeploy.url(args, options),
    method: 'post',
})

retryDeploy.definition = {
    methods: ["post"],
    url: '/wallets/{wallet}/retry',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\WalletController::retryDeploy
* @see app/Http/Controllers/WalletController.php:68
* @route '/wallets/{wallet}/retry'
*/
retryDeploy.url = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions) => {
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

    return retryDeploy.definition.url
            .replace('{wallet}', parsedArgs.wallet.toString())
            .replace(/\/+$/, '') + queryParams(options)
}

/**
* @see \App\Http\Controllers\WalletController::retryDeploy
* @see app/Http/Controllers/WalletController.php:68
* @route '/wallets/{wallet}/retry'
*/
retryDeploy.post = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: retryDeploy.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::assignStrategy
* @see app/Http/Controllers/WalletController.php:82
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
* @see app/Http/Controllers/WalletController.php:82
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
* @see app/Http/Controllers/WalletController.php:82
* @route '/wallets/{wallet}/strategies'
*/
assignStrategy.post = (args: { wallet: number | { id: number } } | [wallet: number | { id: number } ] | number | { id: number }, options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: assignStrategy.url(args, options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\WalletController::removeStrategy
* @see app/Http/Controllers/WalletController.php:100
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
* @see app/Http/Controllers/WalletController.php:100
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
* @see app/Http/Controllers/WalletController.php:100
* @route '/wallets/{wallet}/strategies/{strategy}'
*/
removeStrategy.delete = (args: { wallet: number | { id: number }, strategy: number | { id: number } } | [wallet: number | { id: number }, strategy: number | { id: number } ], options?: RouteQueryOptions): RouteDefinition<'delete'> => ({
    url: removeStrategy.url(args, options),
    method: 'delete',
})

const WalletController = { index, store, destroy, retryDeploy, assignStrategy, removeStrategy }

export default WalletController