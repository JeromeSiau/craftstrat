<?php

use App\Models\Strategy;
use App\Models\Trade;
use App\Models\User;
use App\Models\Wallet;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays strategies index page', function () {
    Strategy::factory()->count(3)->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/index', false)
            ->has('strategies.data', 3)
        );
});

it('includes live and paper performance summaries on strategies index', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'won',
        'is_paper' => false,
        'price' => 0.400000,
        'filled_price' => 0.400000,
        'resolved_price' => 1.000000,
        'size_usdc' => 10.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'lost',
        'is_paper' => true,
        'price' => 0.500000,
        'filled_price' => 0.500000,
        'resolved_price' => 0.000000,
        'size_usdc' => 4.000000,
        'executed_at' => now(),
    ]);

    $this->actingAs($this->user)
        ->get(route('strategies.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->where('strategies.data.0.performance_stats.live.total_trades', 1)
            ->where('strategies.data.0.performance_stats.live.win_rate', '1.0000')
            ->where('strategies.data.0.performance_stats.live.total_pnl_usdc', '15.00')
            ->where('strategies.data.0.performance_stats.paper.total_trades', 1)
            ->where('strategies.data.0.performance_stats.paper.win_rate', '0.0000')
            ->where('strategies.data.0.performance_stats.paper.total_pnl_usdc', '-4.00')
        );
});

it('displays create strategy page', function () {
    $this->actingAs($this->user)
        ->get(route('strategies.create'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/create', false)
        );
});

it('stores a new strategy', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'My Strategy',
            'description' => 'A test strategy',
            'mode' => 'form',
            'graph' => [
                'mode' => 'form',
                'conditions' => [],
                'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_usdc' => 50, 'size_mode' => 'fixed', 'order_type' => 'market'],
                'risk' => ['max_trades_per_slot' => 1],
            ],
        ])
        ->assertRedirect(route('strategies.index'));

    expect(Strategy::where('user_id', $this->user->id)->count())->toBe(1);
});

it('validates required fields on store', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [])
        ->assertSessionHasErrors(['name', 'graph', 'mode']);
});

it('shows a strategy belonging to the user', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/show', false)
            ->has('strategy')
        );
});

it('prevents viewing another users strategy', function () {
    $other = User::factory()->create();
    $strategy = Strategy::factory()->create(['user_id' => $other->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertForbidden();
});

it('updates a strategy', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->put(route('strategies.update', $strategy), ['name' => 'Updated Name'])
        ->assertRedirect();

    expect($strategy->fresh()->name)->toBe('Updated Name');
});

it('deletes a strategy', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->delete(route('strategies.destroy', $strategy))
        ->assertRedirect(route('strategies.index'));

    expect(Strategy::find($strategy->id))->toBeNull();
});

it('activates a strategy and calls engine', function () {
    Http::fake(['*/internal/strategy/activate' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.activate', $strategy))
        ->assertRedirect();

    expect($strategy->fresh()->is_active)->toBeTrue();
});

it('rolls back engine activation when a later assignment fails', function () {
    Http::fake([
        '*/internal/strategy/activate' => Http::sequence()
            ->push(null, 200)
            ->push(null, 500),
        '*/internal/strategy/deactivate' => Http::response(null, 200),
    ]);

    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $firstWallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $secondWallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $strategy->walletStrategies()->create([
        'wallet_id' => $firstWallet->id,
        'markets' => ['BTC'],
        'max_position_usdc' => 100,
        'is_paper' => false,
        'is_running' => false,
    ]);

    $strategy->walletStrategies()->create([
        'wallet_id' => $secondWallet->id,
        'markets' => ['ETH'],
        'max_position_usdc' => 150,
        'is_paper' => true,
        'is_running' => false,
    ]);

    $this->actingAs($this->user)
        ->from(route('strategies.show', $strategy))
        ->post(route('strategies.activate', $strategy))
        ->assertRedirect(route('strategies.show', $strategy))
        ->assertSessionHas('error');

    expect($strategy->fresh()->is_active)->toBeFalse()
        ->and($strategy->walletStrategies()->where('is_running', true)->count())->toBe(0);

    Http::assertSent(fn ($request) => str_ends_with($request->url(), '/internal/strategy/deactivate')
        && $request['wallet_id'] === $firstWallet->id
        && $request['strategy_id'] === $strategy->id
    );
});

it('shows a friendly error when an assigned wallet is not deployed', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->pending()->create(['user_id' => $this->user->id]);

    $strategy->walletStrategies()->create([
        'wallet_id' => $wallet->id,
        'markets' => ['BTC'],
        'max_position_usdc' => 100,
        'is_paper' => false,
        'is_running' => false,
    ]);

    $this->actingAs($this->user)
        ->from(route('strategies.show', $strategy))
        ->post(route('strategies.activate', $strategy))
        ->assertRedirect(route('strategies.show', $strategy))
        ->assertSessionHas('error', "Wallet #{$wallet->id} Safe is not deployed yet.");
});

it('deactivates a strategy and calls engine', function () {
    Http::fake(['*/internal/strategy/deactivate' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.deactivate', $strategy))
        ->assertRedirect();

    expect($strategy->fresh()->is_active)->toBeFalse();
});

it('rolls back engine deactivation when a later assignment fails', function () {
    Http::fake([
        '*/internal/strategy/deactivate' => Http::sequence()
            ->push(null, 200)
            ->push(null, 500),
        '*/internal/strategy/activate' => Http::response(null, 200),
    ]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);
    $firstWallet = Wallet::factory()->create(['user_id' => $this->user->id]);
    $secondWallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    $strategy->walletStrategies()->create([
        'wallet_id' => $firstWallet->id,
        'markets' => ['BTC'],
        'max_position_usdc' => 100,
        'is_paper' => false,
        'is_running' => true,
        'started_at' => now(),
    ]);

    $strategy->walletStrategies()->create([
        'wallet_id' => $secondWallet->id,
        'markets' => ['ETH'],
        'max_position_usdc' => 150,
        'is_paper' => true,
        'is_running' => true,
        'started_at' => now(),
    ]);

    $this->actingAs($this->user)
        ->from(route('strategies.show', $strategy))
        ->post(route('strategies.deactivate', $strategy))
        ->assertRedirect(route('strategies.show', $strategy))
        ->assertSessionHas('error');

    expect($strategy->fresh()->is_active)->toBeTrue()
        ->and($strategy->walletStrategies()->where('is_running', true)->count())->toBe(2);

    Http::assertSent(fn ($request) => str_ends_with($request->url(), '/internal/strategy/activate')
        && $request['wallet_id'] === $firstWallet->id
        && $request['strategy_id'] === $strategy->id
    );
});

it('loads deferred live stats and recent trades', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->count(5)->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'filled',
    ]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('strategies/show', false)
            ->has('strategy')
            ->missing('liveStats')
            ->missing('recentTrades')
            ->loadDeferredProps('liveData', fn (Assert $reload) => $reload
                ->has('liveStats.live', fn (Assert $stats) => $stats
                    ->where('total_trades', 5)
                    ->has('win_rate')
                    ->has('total_pnl_usdc')
                    ->etc()
                )
                ->has('liveStats.paper')
                ->has('recentTrades', 5)
            )
        );
});

it('builds paper stats from resolved trades', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'won',
        'is_paper' => true,
        'price' => 0.400000,
        'filled_price' => 0.400000,
        'resolved_price' => 1.000000,
        'size_usdc' => 10.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'lost',
        'is_paper' => true,
        'price' => 0.250000,
        'filled_price' => 0.250000,
        'resolved_price' => 0.000000,
        'size_usdc' => 4.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'filled',
        'is_paper' => true,
        'price' => 0.550000,
        'filled_price' => 0.550000,
        'size_usdc' => 2.000000,
        'executed_at' => now(),
    ]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->loadDeferredProps('liveData', fn (Assert $reload) => $reload
                ->where('liveStats.paper.total_trades', 3)
                ->where('liveStats.paper.win_rate', '0.5000')
                ->where('liveStats.paper.total_pnl_usdc', '11.00')
            )
        );
});

it('builds paper stats from closed buy trades and ignores sell exits', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'closed',
        'is_paper' => true,
        'price' => 0.400000,
        'filled_price' => 0.400000,
        'resolved_price' => 0.550000,
        'size_usdc' => 10.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'buy',
        'status' => 'closed',
        'is_paper' => true,
        'price' => 0.600000,
        'filled_price' => 0.600000,
        'resolved_price' => 0.450000,
        'size_usdc' => 6.000000,
        'executed_at' => now(),
    ]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'side' => 'sell',
        'status' => 'won',
        'is_paper' => true,
        'price' => 0.550000,
        'filled_price' => 0.550000,
        'resolved_price' => 1.000000,
        'size_usdc' => 10.000000,
        'executed_at' => now(),
    ]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->loadDeferredProps('liveData', fn (Assert $reload) => $reload
                ->where('liveStats.paper.total_trades', 2)
                ->where('liveStats.paper.win_rate', '0.5000')
                ->where('liveStats.paper.total_pnl_usdc', '2.25')
            )
        );
});

it('includes filled price in recent trades payload', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
        'status' => 'filled',
        'price' => null,
        'filled_price' => 0.612345,
        'executed_at' => now(),
    ]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->loadDeferredProps('liveData', fn (Assert $reload) => $reload
                ->where('recentTrades.0.filled_price', '0.612345')
            )
        );
});

it('returns empty trades when strategy has no trades', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->get(route('strategies.show', $strategy))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->loadDeferredProps('liveData', fn (Assert $reload) => $reload
                ->where('liveStats.live.total_trades', 0)
                ->where('liveStats.live.win_rate', null)
                ->where('liveStats.paper.total_trades', 0)
                ->has('recentTrades', 0)
            )
        );
});

it('requires authentication', function () {
    $this->get(route('strategies.index'))->assertRedirect('/login');
});
