<?php

namespace App\Http\Controllers;

use App\Http\Requests\SubscribeRequest;
use App\Services\BillingService;
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
            'subscribed' => $user->subscribed('default'),
        ]);
    }

    public function subscribe(SubscribeRequest $request, BillingService $billing): RedirectResponse
    {
        return $billing->checkout($request->user(), $request->validated('price_id'))
            ->redirect();
    }

    public function portal(Request $request, BillingService $billing): RedirectResponse
    {
        return $billing->billingPortalRedirect($request->user());
    }
}
