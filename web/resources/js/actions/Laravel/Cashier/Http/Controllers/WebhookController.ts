import { queryParams, type RouteQueryOptions, type RouteDefinition, type RouteFormDefinition } from './../../../../../wayfinder'
/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/stripe/webhook'
*/
const handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678 = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.url(options),
    method: 'post',
})

handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.definition = {
    methods: ["post"],
    url: '/stripe/webhook',
} satisfies RouteDefinition<["post"]>

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/stripe/webhook'
*/
handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.url = (options?: RouteQueryOptions) => {
    return handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.definition.url + queryParams(options)
}

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/stripe/webhook'
*/
handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.url(options),
    method: 'post',
})

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/stripe/webhook'
*/
const handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678Form = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.url(options),
    method: 'post',
})

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/stripe/webhook'
*/
handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678Form.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.url(options),
    method: 'post',
})

handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678.form = handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678Form
/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/webhooks/stripe'
*/
const handleWebhook10d7cc18815c3fd03e906610c643475f = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: handleWebhook10d7cc18815c3fd03e906610c643475f.url(options),
    method: 'post',
})

handleWebhook10d7cc18815c3fd03e906610c643475f.definition = {
    methods: ["post"],
    url: '/webhooks/stripe',
} satisfies RouteDefinition<["post"]>

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/webhooks/stripe'
*/
handleWebhook10d7cc18815c3fd03e906610c643475f.url = (options?: RouteQueryOptions) => {
    return handleWebhook10d7cc18815c3fd03e906610c643475f.definition.url + queryParams(options)
}

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/webhooks/stripe'
*/
handleWebhook10d7cc18815c3fd03e906610c643475f.post = (options?: RouteQueryOptions): RouteDefinition<'post'> => ({
    url: handleWebhook10d7cc18815c3fd03e906610c643475f.url(options),
    method: 'post',
})

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/webhooks/stripe'
*/
const handleWebhook10d7cc18815c3fd03e906610c643475fForm = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: handleWebhook10d7cc18815c3fd03e906610c643475f.url(options),
    method: 'post',
})

/**
* @see \Laravel\Cashier\Http\Controllers\WebhookController::handleWebhook
* @see vendor/laravel/cashier/src/Http/Controllers/WebhookController.php:40
* @route '/webhooks/stripe'
*/
handleWebhook10d7cc18815c3fd03e906610c643475fForm.post = (options?: RouteQueryOptions): RouteFormDefinition<'post'> => ({
    action: handleWebhook10d7cc18815c3fd03e906610c643475f.url(options),
    method: 'post',
})

handleWebhook10d7cc18815c3fd03e906610c643475f.form = handleWebhook10d7cc18815c3fd03e906610c643475fForm

export const handleWebhook = {
    '/stripe/webhook': handleWebhooke8624cfbbb7b45dd73ec39b90a7c4678,
    '/webhooks/stripe': handleWebhook10d7cc18815c3fd03e906610c643475f,
}

const WebhookController = { handleWebhook }

export default WebhookController