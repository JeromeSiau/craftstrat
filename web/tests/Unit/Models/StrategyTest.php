<?php

use App\Models\Strategy;
use App\Models\User;
use Illuminate\Database\Eloquent\Relations\HasMany;

uses(Tests\TestCase::class, Illuminate\Foundation\Testing\RefreshDatabase::class);

it('belongs to a user', function () {
    $strategy = Strategy::factory()->create();

    expect($strategy->user)->toBeInstanceOf(User::class);
});

it('casts graph as array', function () {
    $strategy = Strategy::factory()->create();

    expect($strategy->graph)->toBeArray()
        ->and($strategy->graph['mode'])->toBe('form');
});

it('casts is_active as boolean', function () {
    $strategy = Strategy::factory()->active()->create();

    expect($strategy->is_active)->toBeTrue();
});

it('defines a backtest results relationship', function () {
    $strategy = Strategy::factory()->create();

    expect($strategy->backtestResults())->toBeInstanceOf(HasMany::class);
})->skip(! class_exists(\App\Models\BacktestResult::class), 'BacktestResult model not yet created');
