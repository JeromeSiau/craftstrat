<?php

namespace App\Http\Controllers;

use Illuminate\Http\RedirectResponse;
use Illuminate\Http\Request;
use Inertia\Inertia;
use Inertia\Response;

class BillingController extends Controller
{
    public function index(Request $request): Response
    {
        $user = $request->user();

        return Inertia::render('billing/index', [
            'plan' => $user->plan ?? 'free',
            'subscription' => $user->subscription('default'),
            'onTrial' => $user->onTrial('default'),
            'subscribed' => $user->subscribed('default'),
        ]);
    }

    public function subscribe(Request $request): RedirectResponse
    {
        $validated = $request->validate([
            'price_id' => ['required', 'string'],
        ]);

        return $request->user()
            ->newSubscription('default', $validated['price_id'])
            ->checkout([
                'success_url' => route('billing.index').'?checkout=success',
                'cancel_url' => route('billing.index').'?checkout=cancelled',
            ])
            ->redirect();
    }

    public function portal(Request $request): RedirectResponse
    {
        return $request->user()->redirectToBillingPortal(route('billing.index'));
    }
}
