<?php

use App\Http\Controllers\BacktestController;
use App\Http\Controllers\BillingController;
use App\Http\Controllers\DashboardController;
use App\Http\Controllers\StrategyController;
use App\Http\Controllers\WalletController;
use Illuminate\Support\Facades\Route;
use Inertia\Inertia;
use Laravel\Fortify\Features;

Route::get('/', function () {
    return Inertia::render('welcome', [
        'canRegister' => Features::enabled(Features::registration()),
    ]);
})->name('home');

Route::middleware(['auth', 'verified'])->group(function () {
    Route::get('dashboard', [DashboardController::class, 'index'])->name('dashboard');

    // Strategies
    Route::resource('strategies', StrategyController::class)->except(['edit', 'store']);
    Route::post('strategies', [StrategyController::class, 'store'])->name('strategies.store')->middleware('plan.limit:strategies');
    Route::post('strategies/{strategy}/activate', [StrategyController::class, 'activate'])->name('strategies.activate');
    Route::post('strategies/{strategy}/deactivate', [StrategyController::class, 'deactivate'])->name('strategies.deactivate');

    // Wallets
    Route::get('wallets', [WalletController::class, 'index'])->name('wallets.index');
    Route::post('wallets', [WalletController::class, 'store'])->name('wallets.store')->middleware('plan.limit:wallets');
    Route::delete('wallets/{wallet}', [WalletController::class, 'destroy'])->name('wallets.destroy');
    Route::post('wallets/{wallet}/strategies', [WalletController::class, 'assignStrategy'])->name('wallets.assign-strategy');
    Route::delete('wallets/{wallet}/strategies/{strategy}', [WalletController::class, 'removeStrategy'])->name('wallets.remove-strategy');

    // Backtests
    Route::get('backtests', [BacktestController::class, 'index'])->name('backtests.index');
    Route::get('backtests/{result}', [BacktestController::class, 'show'])->name('backtests.show');
    Route::post('strategies/{strategy}/backtest', [BacktestController::class, 'run'])->name('backtests.run');

    // Billing
    Route::get('billing', [BillingController::class, 'index'])->name('billing.index');
    Route::post('billing/subscribe', [BillingController::class, 'subscribe'])->name('billing.subscribe');
    Route::post('billing/portal', [BillingController::class, 'portal'])->name('billing.portal');
});

// Stripe Webhook (no auth)
Route::post('webhooks/stripe', [\Laravel\Cashier\Http\Controllers\WebhookController::class, 'handleWebhook'])
    ->name('cashier.webhook');

require __DIR__.'/settings.php';
