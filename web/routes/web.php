<?php

use App\Http\Controllers\AnalyticsController;
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
    Route::post('strategies/generate', [StrategyController::class, 'generate'])->name('strategies.generate')->middleware('throttle:ai-generation');
    Route::resource('strategies', StrategyController::class)->except(['edit', 'store']);
    Route::post('strategies', [StrategyController::class, 'store'])->name('strategies.store')->middleware('plan.limit:strategies');
    Route::post('strategies/{strategy}/activate', [StrategyController::class, 'activate'])->name('strategies.activate');
    Route::post('strategies/{strategy}/deactivate', [StrategyController::class, 'deactivate'])->name('strategies.deactivate');
    Route::post('strategies/{strategy}/kill', [StrategyController::class, 'kill'])->name('strategies.kill');
    Route::post('strategies/{strategy}/unkill', [StrategyController::class, 'unkill'])->name('strategies.unkill');

    // Wallets
    Route::get('wallets', [WalletController::class, 'index'])->name('wallets.index');
    Route::post('wallets', [WalletController::class, 'store'])->name('wallets.store')->middleware('plan.limit:wallets');
    Route::delete('wallets/{wallet}', [WalletController::class, 'destroy'])->name('wallets.destroy');
    Route::post('wallets/{wallet}/retry', [WalletController::class, 'retryDeploy'])->name('wallets.retry');
    Route::post('wallets/{wallet}/strategies', [WalletController::class, 'assignStrategy'])->name('wallets.assign-strategy');
    Route::delete('wallets/{wallet}/strategies/{strategy}', [WalletController::class, 'removeStrategy'])->name('wallets.remove-strategy');

    // Backtests
    Route::get('backtests', [BacktestController::class, 'index'])->name('backtests.index');
    Route::get('backtests/{result}', [BacktestController::class, 'show'])->name('backtests.show');
    Route::delete('backtests/{result}', [BacktestController::class, 'destroy'])->name('backtests.destroy');
    Route::post('backtests/{result}/rerun', [BacktestController::class, 'rerun'])->name('backtests.rerun');
    Route::post('strategies/{strategy}/backtest', [BacktestController::class, 'run'])->name('backtests.run');

    // Billing
    Route::get('billing', [BillingController::class, 'index'])->name('billing.index');
    Route::post('billing/subscribe', [BillingController::class, 'subscribe'])->name('billing.subscribe');
    Route::post('billing/portal', [BillingController::class, 'portal'])->name('billing.portal');

    // Analytics
    Route::get('analytics', [AnalyticsController::class, 'index'])->name('analytics.index');
});

// Internal API (engine → Laravel, no auth)
Route::post('internal/notification/send', [\App\Http\Controllers\InternalNotificationController::class, 'send'])
    ->name('internal.notification.send');

// Stripe Webhook (no auth)
Route::post('webhooks/stripe', [\Laravel\Cashier\Http\Controllers\WebhookController::class, 'handleWebhook'])
    ->name('cashier.webhook');

require __DIR__.'/settings.php';
