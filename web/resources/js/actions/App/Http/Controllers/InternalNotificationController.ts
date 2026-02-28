import { queryParams, type RouteQueryOptions, type RouteDefinition } from './../../../../wayfinder'
/**
* @see \App\Http\Controllers\InternalNotificationController::send
* @see app/Http/Controllers/InternalNotificationController.php:12
* @route '/internal/notification/send'
*/
export const send = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: send.url(options),
    method: 'post',
})

send.definition = {
    methods: ["post"],
    url: '/internal/notification/send',
} satisfies RouteDefinition<["post"]>

/**
* @see \App\Http\Controllers\InternalNotificationController::send
* @see app/Http/Controllers/InternalNotificationController.php:12
* @route '/internal/notification/send'
*/
send.url = (options?: RouteQueryOptions) => {
    return send.definition.url + queryParams(options)
}

/**
* @see \App\Http\Controllers\InternalNotificationController::send
* @see app/Http/Controllers/InternalNotificationController.php:12
* @route '/internal/notification/send'
*/
send.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: send.url(options),
    method: 'post',
})

const InternalNotificationController = { send }

export default InternalNotificationController