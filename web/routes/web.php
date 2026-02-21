<?php

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
    Route::get('dashboard', function () {
        return Inertia::render('dashboard');
    })->name('dashboard');

    // Strategies
    Route::resource('strategies', StrategyController::class)->except(['edit']);
    Route::post('strategies/{strategy}/activate', [StrategyController::class, 'activate'])->name('strategies.activate');
    Route::post('strategies/{strategy}/deactivate', [StrategyController::class, 'deactivate'])->name('strategies.deactivate');

    // Wallets
    Route::get('wallets', [WalletController::class, 'index'])->name('wallets.index');
    Route::post('wallets', [WalletController::class, 'store'])->name('wallets.store');
    Route::delete('wallets/{wallet}', [WalletController::class, 'destroy'])->name('wallets.destroy');
    Route::post('wallets/{wallet}/strategies', [WalletController::class, 'assignStrategy'])->name('wallets.assign-strategy');
    Route::delete('wallets/{wallet}/strategies/{strategy}', [WalletController::class, 'removeStrategy'])->name('wallets.remove-strategy');
});

require __DIR__.'/settings.php';
