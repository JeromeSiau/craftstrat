import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition } from './../../../../wayfinder'
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

/**
* @see \App\Http\Controllers\InternalNotificationController::send
* @see app/Http/Controllers/InternalNotificationController.php:12
* @route '/internal/notification/send'
*/
const sendForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: send.url(options),
    method: 'post',
})

/**
* @see \App\Http\Controllers\InternalNotificationController::send
* @see app/Http/Controllers/InternalNotificationController.php:12
* @route '/internal/notification/send'
*/
sendForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: send.url(options),
    method: 'post',
})

send.form = sendForm

const InternalNotificationController = { send }

export default InternalNotificationController