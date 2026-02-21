<?php

return [

    /*
    |--------------------------------------------------------------------------
    | Allowed Stripe Price IDs
    |--------------------------------------------------------------------------
    |
    | Only these Stripe price IDs can be used for subscriptions.
    | Add your actual Stripe price IDs from the Stripe Dashboard.
    |
    */

    'allowed_price_ids' => array_filter(explode(',', env('STRIPE_ALLOWED_PRICE_IDS', ''))),

];
