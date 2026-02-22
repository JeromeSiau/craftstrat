<?php

use App\Models\User;
use Illuminate\Support\Facades\Http;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays analytics page with stats from engine', function () {
    Http::fake(['*/internal/stats/slots*' => Http::response([
        'summary' => [
            'total_slots' => 100,
            'resolved_slots' => 90,
            'unresolved_slots' => 10,
            'total_snapshots' => 5000,
            'last_snapshot_at' => '2026-02-22T12:00:00Z',
        ],
        'heatmap' => [],
        'calibration' => [],
        'by_symbol' => [
            ['symbol' => 'BTC', 'total' => 50, 'wins' => 26, 'win_rate' => 52.0],
        ],
        'stoploss_sweep' => [],
        'by_hour' => [],
        'by_day' => [],
    ])]);

    $this->actingAs($this->user)
        ->get(route('analytics.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('analytics/index', false)
            ->has('stats')
            ->has('filters')
        );
});

it('passes filters to the engine', function () {
    Http::fake(['*/internal/stats/slots*' => Http::response([
        'summary' => ['total_slots' => 0, 'resolved_slots' => 0, 'unresolved_slots' => 0, 'total_snapshots' => 0, 'last_snapshot_at' => null],
        'heatmap' => [], 'calibration' => [], 'by_symbol' => [],
        'stoploss_sweep' => [], 'by_hour' => [], 'by_day' => [],
    ])]);

    $this->actingAs($this->user)
        ->get(route('analytics.index', ['slot_duration' => 300, 'symbols' => 'BTC,ETH']))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->where('filters.slot_duration', 300)
            ->where('filters.symbols', ['BTC', 'ETH'])
        );

    Http::assertSent(fn ($request) => str_contains($request->url(), 'slot_duration=300')
        && str_contains($request->url(), 'symbols=BTC%2CETH')
    );
});

it('handles engine errors gracefully', function () {
    Http::fake(['*/internal/stats/slots*' => Http::response([], 500)]);

    $this->actingAs($this->user)
        ->get(route('analytics.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('analytics/index', false)
            ->where('stats', null)
        );
});

it('requires authentication', function () {
    $this->get(route('analytics.index'))->assertRedirect('/login');
});
