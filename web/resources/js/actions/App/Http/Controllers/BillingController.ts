import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition } from './../../../../wayfinder'
/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
*/
export const index = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

index.definition = {
    methods: ["get","head"],
    url: '/billing',
} satisfies RouteDefinition<["get","head"]>

/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
*/
index.url = (options?: RouteQueryOptions) => {
    return index.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
*/
index.get = (options?: RouteQueryOptions): RouteDefinition<'get'> => ({
    url: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
*/
index.head = (options?: RouteQueryOptions): RouteDefinition<'head'> => ({
    url: index.url(options),
    method: 'head',
})

/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
*/
const indexForm = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
*/
indexForm.get = (options?: RouteQueryOptions): RouteFormDefinition<'get'> => ({
    action: index.url(options),
    method: 'get',
})

/**
* @see \App\Http\Controllers\BillingController::index
* @see app/Http/Controllers/BillingController.php:14
* @route '/billing'
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
* @see \App\Http\Controllers\BillingController::subscribe
* @see app/Http/Controllers/BillingController.php:24
* @route '/billing/subscribe'
*/
export const subscribe = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: subscribe.url(options),
    method: 'post',
})

subscribe.definition = {
    methods: ["post"],
    url: '/billing/subscribe',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\BillingController::subscribe
* @see app/Http/Controllers/BillingController.php:24
* @route '/billing/subscribe'
*/
subscribe.url = (options?: RouteQueryOptions) => {
    return subscribe.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\BillingController::subscribe
* @see app/Http/Controllers/BillingController.php:24
* @route '/billing/subscribe'
*/
subscribe.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: subscribe.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\BillingController::subscribe
* @see app/Http/Controllers/BillingController.php:24
* @route '/billing/subscribe'
*/
const subscribeForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: subscribe.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\BillingController::subscribe
* @see app/Http/Controllers/BillingController.php:24
* @route '/billing/subscribe'
*/
subscribeForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: subscribe.url(options),
    method: 'post',
})

subscribe.form = subscribeForm

/**
* @see \App\Http\Controllers\BillingController::portal
* @see app/Http/Controllers/BillingController.php:30
* @route '/billing/portal'
*/
export const portal = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: portal.url(options),
    method: 'post',
})

portal.definition = {
    methods: ["post"],
    url: '/billing/portal',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\BillingController::portal
* @see app/Http/Controllers/BillingController.php:30
* @route '/billing/portal'
*/
portal.url = (options?: RouteQueryOptions) => {
    return portal.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\BillingController::portal
* @see app/Http/Controllers/BillingController.php:30
* @route '/billing/portal'
*/
portal.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: portal.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\BillingController::portal
* @see app/Http/Controllers/BillingController.php:30
* @route '/billing/portal'
*/
const portalForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: portal.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\BillingController::portal
* @see app/Http/Controllers/BillingController.php:30
* @route '/billing/portal'
*/
portalForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: portal.url(options),
    method: 'post',
})

portal.form = portalForm

const BillingController = { index, subscribe, portal }

export default BillingController