<?php

use App\Models\User;

beforeEach(function () {
    $this->user = User::factory()->create();
});

function nodeGraphWithApiFetch(array $apiFetchNodes): array
{
    $nodes = [];
    foreach ($apiFetchNodes as $i => $data) {
        $nodes[] = [
            'id' => "n{$i}",
            'type' => 'api_fetch',
            'data' => $data,
        ];
    }

    return [
        'name' => 'API Strategy',
        'mode' => 'node',
        'graph' => [
            'mode' => 'node',
            'nodes' => $nodes,
            'edges' => [],
        ],
    ];
}

it('accepts valid api_fetch nodes', function () {
    $payload = nodeGraphWithApiFetch([
        ['url' => 'https://api.example.com/data', 'json_path' => 'main.temp', 'interval_secs' => 60],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertRedirect(route('strategies.index'));
});

it('rejects non-HTTPS api_fetch URLs', function () {
    $payload = nodeGraphWithApiFetch([
        ['url' => 'http://api.example.com/data', 'json_path' => 'main.temp', 'interval_secs' => 60],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects localhost api_fetch URLs', function () {
    $payload = nodeGraphWithApiFetch([
        ['url' => 'https://localhost/data', 'json_path' => 'main.temp', 'interval_secs' => 60],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects private IP api_fetch URLs', function () {
    $payload = nodeGraphWithApiFetch([
        ['url' => 'https://192.168.1.1/data', 'json_path' => 'main.temp', 'interval_secs' => 60],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects api_fetch interval below 30 seconds', function () {
    $payload = nodeGraphWithApiFetch([
        ['url' => 'https://api.example.com/data', 'json_path' => 'main.temp', 'interval_secs' => 10],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects more than 5 api_fetch nodes', function () {
    $nodes = [];
    for ($i = 0; $i < 6; $i++) {
        $nodes[] = ['url' => "https://api.example.com/data{$i}", 'json_path' => 'value', 'interval_secs' => 60];
    }
    $payload = nodeGraphWithApiFetch($nodes);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('allows exactly 5 api_fetch nodes', function () {
    $nodes = [];
    for ($i = 0; $i < 5; $i++) {
        $nodes[] = ['url' => "https://api.example.com/data{$i}", 'json_path' => 'value', 'interval_secs' => 60];
    }
    $payload = nodeGraphWithApiFetch($nodes);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertRedirect(route('strategies.index'));
});

it('rejects api_fetch nodes with empty URL', function () {
    $payload = nodeGraphWithApiFetch([
        ['url' => '', 'json_path' => 'main.temp', 'interval_secs' => 60],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('skips api_fetch validation for form mode strategies', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'Form Strategy',
            'mode' => 'form',
            'graph' => [
                'mode' => 'form',
                'conditions' => [],
                'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_usdc' => 50, 'size_mode' => 'fixed', 'order_type' => 'market'],
                'risk' => ['max_trades_per_slot' => 1],
            ],
        ])
        ->assertRedirect(route('strategies.index'));
});

it('rejects form limit orders without a valid limit price', function () {
    $this->actingAs($this->user)
        ->post(route('strategies.store'), [
            'name' => 'Form Limit Strategy',
            'mode' => 'form',
            'graph' => [
                'mode' => 'form',
                'conditions' => [
                    [
                        'type' => 'AND',
                        'rules' => [
                            ['indicator' => 'abs_move_pct', 'operator' => '>', 'value' => 1],
                        ],
                    ],
                ],
                'action' => [
                    'signal' => 'buy',
                    'outcome' => 'UP',
                    'size_usdc' => 50,
                    'size_mode' => 'fixed',
                    'order_type' => 'limit',
                ],
                'risk' => ['max_trades_per_slot' => 1],
            ],
        ])
        ->assertSessionHasErrors(['graph.action.limit_price']);
});

it('rejects node limit orders without a valid limit price', function () {
    $payload = [
        'name' => 'Node Limit Strategy',
        'mode' => 'node',
        'graph' => [
            'mode' => 'node',
            'nodes' => [
                [
                    'id' => 'n1',
                    'type' => 'action',
                    'data' => [
                        'signal' => 'buy',
                        'outcome' => 'UP',
                        'size_usdc' => 50,
                        'order_type' => 'limit',
                    ],
                ],
            ],
            'edges' => [],
        ],
    ];

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors(['graph.nodes.0.data.limit_price']);
});
