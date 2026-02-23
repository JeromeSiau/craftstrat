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

it('deactivates a strategy and calls engine', function () {
    Http::fake(['*/internal/strategy/deactivate' => Http::response(null, 200)]);

    $strategy = Strategy::factory()->active()->create(['user_id' => $this->user->id]);

    $this->actingAs($this->user)
        ->post(route('strategies.deactivate', $strategy))
        ->assertRedirect();

    expect($strategy->fresh()->is_active)->toBeFalse();
});

it('loads deferred live stats and recent trades', function () {
    $strategy = Strategy::factory()->create(['user_id' => $this->user->id]);
    $wallet = Wallet::factory()->create(['user_id' => $this->user->id]);

    Trade::factory()->count(5)->create([
        'wallet_id' => $wallet->id,
        'strategy_id' => $strategy->id,
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
                )
                ->has('liveStats.paper')
                ->has('recentTrades', 5)
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
