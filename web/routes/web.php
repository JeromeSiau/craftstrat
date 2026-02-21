<?php

use App\Http\Controllers\StrategyController;
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
});

require __DIR__.'/settings.php';
