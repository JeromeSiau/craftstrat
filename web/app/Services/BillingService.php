<?php

namespace App\Services;

use App\Models\User;
use Illuminate\Http\RedirectResponse;
use Laravel\Cashier\Checkout;

class BillingService
{
    public function checkout(User $user, string $priceId): Checkout
    {
        return $user->newSubscription('default', $priceId)
            ->checkout([
                'success_url' => route('billing.index').'?checkout=success',
                'cancel_url' => route('billing.index').'?checkout=cancelled',
            ]);
    }

    public function billingPortalRedirect(User $user): RedirectResponse
    {
        return $user->redirectToBillingPortal(route('billing.index'));
    }
}
