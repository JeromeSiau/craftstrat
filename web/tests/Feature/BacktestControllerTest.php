<?php

use App\Models\BacktestResult;
use App\Models\Strategy;
use App\Models\User;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays backtests index page', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    BacktestResult::factory()->count(2)->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('backtests/index', false)
            ->has('results.data', 2)
        );
});

it('shows a backtest result', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.show', $result))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('backtests/show', false)
            ->has('result')
        );
});

it('prevents viewing another users backtest result', function () {
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $other->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $other->id,
        'strategy_id' => $strategy->id,
    ]);

    $this->actingAs($this->user)
        ->get(route('backtests.show', $result))
        ->assertForbidden();
});

it('runs a backtest via the engine and stores the result', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 42,
        'win_rate' => 0.65,
        'total_pnl_usdc' => 123.45,
        'max_drawdown' => 0.12,
        'sharpe_ratio' => 1.5,
        'trades' => [],
    ])]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(7)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertRedirect();

    $result = BacktestResult::where('strategy_id', $strategy->id)->first();
    expect($result)->not->toBeNull()
        ->and($result->total_trades)->toBe(42)
        ->and($result->win_rate)->toBe('0.6500');
});

it('validates backtest request fields', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [])
        ->assertSessionHasErrors(['date_from'])
        ->assertSessionDoesntHaveErrors(['date_to']);
});

it('defaults date_to to today when left blank', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 3,
        'win_rate' => 0.67,
        'total_pnl_usdc' => 12.5,
        'max_drawdown' => 0.03,
        'sharpe_ratio' => 1.1,
        'trades' => [],
    ])]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $today = now()->toDateString();
    $expectedDateTo = now()->endOfDay()->toIso8601ZuluString();

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(7)->toDateString(),
            'date_to' => '',
        ])
        ->assertRedirect();

    $result = BacktestResult::where('strategy_id', $strategy->id)->first();

    expect($result)->not->toBeNull()
        ->and($result->date_to->toDateString())->toBe($today);

    Http::assertSent(fn ($request) => str_ends_with($request->url(), '/internal/backtest/run')
        && $request['date_to'] === $expectedDateTo
    );
});

it('enforces backtest_days limit for free plan', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(60)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertSessionHasErrors(['date_from']);
});

it('allows full history backtest for starter plan', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 10,
        'win_rate' => 0.50,
        'total_pnl_usdc' => 50.0,
        'max_drawdown' => 0.05,
        'sharpe_ratio' => 1.0,
        'trades' => [],
    ])]);

    $user = User::factory()->create(['plan' => 'starter']);
    $strategy = Strategy::factory()->create(['user_id' => $user->id]);

    $this->actingAs($user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(365)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertRedirect();
});

it('transforms engine trades when storing a backtest result', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 2,
        'win_rate' => 0.50,
        'total_pnl_usdc' => 1.25,
        'max_drawdown' => 0.05,
        'sharpe_ratio' => 1.1,
        'trades' => [
            [
                'side' => 'buy',
                'outcome' => 'up',
                'entry_price' => 0.40,
                'entry_reference_price' => 0.39,
                'entry_slippage_bps' => 1.234,
                'entry_book_depth_usdc' => 100.123456,
                'entry_depth_ratio' => 0.3333333,
                'exit_price' => 0.46,
                'exit_reference_price' => 0.45,
                'exit_slippage_bps' => -2.345,
                'exit_book_depth_usdc' => 90.54321,
                'exit_depth_ratio' => 0.2222222,
                'pnl_usdc' => 1.5,
                'symbol' => 'btc-updown-15m-1',
                'entry_at' => '2026-03-01T00:00:00Z',
                'exit_at' => '2026-03-01T00:15:00Z',
                'exit_reason' => 'take_profit',
            ],
            [
                'side' => 'sell',
                'outcome' => 'down',
                'entry_price' => 0.55,
                'pnl_usdc' => -0.25,
            ],
        ],
    ])]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('backtests.run', $strategy), [
            'date_from' => now()->subDays(7)->toDateString(),
            'date_to' => now()->toDateString(),
        ])
        ->assertRedirect();

    $result = BacktestResult::where('strategy_id', $strategy->id)->first();

    expect($result)->not->toBeNull()
        ->and($result->result_detail['trades'][0]['outcome'])->toBe('UP')
        ->and($result->result_detail['trades'][0]['entry_slippage_bps'])->toBe(1.23)
        ->and($result->result_detail['trades'][0]['entry_book_depth_usdc'])->toBe(100.1235)
        ->and($result->result_detail['trades'][0]['entry_depth_ratio'])->toBe(0.333333)
        ->and($result->result_detail['trades'][0]['exit_slippage_bps'])->toBe(-2.35)
        ->and($result->result_detail['trades'][0]['cumulative_pnl'])->toBe(1.5)
        ->and($result->result_detail['trades'][1]['cumulative_pnl'])->toBe(1.25);
});

it('reruns a backtest and refreshes the stored result detail', function () {
    Http::fake(['*/internal/backtest/run' => Http::response([
        'total_trades' => 1,
        'win_rate' => 1.00,
        'total_pnl_usdc' => 4.5,
        'max_drawdown' => 0.01,
        'sharpe_ratio' => 2.2,
        'trades' => [[
            'side' => 'buy',
            'outcome' => 'up',
            'entry_price' => 0.42,
            'pnl_usdc' => 4.5,
        ]],
    ])]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $result = BacktestResult::factory()->create([
        'user_id' => $this->user->id,
        'strategy_id' => $strategy->id,
        'result_detail' => ['trades' => [['cumulative_pnl' => 0.0]]],
    ]);

    $this->actingAs($this->user)
        ->from(route('backtests.show', $result))
        ->post(route('backtests.rerun', $result))
        ->assertRedirect(route('backtests.show', $result));

    $result->refresh();

    expect($result->total_trades)->toBe(1)
        ->and($result->total_pnl_usdc)->toBe('4.500000')
        ->and($result->result_detail['trades'][0]['cumulative_pnl'])->toBe(4.5)
        ->and($result->result_detail['trades'][0]['outcome'])->toBe('UP');
});

it('requires authentication', function () {
    $this->get(route('backtests.index'))->assertRedirect('/login');
});
