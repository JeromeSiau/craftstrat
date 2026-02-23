<?php

use App\Models\User;
use Illuminate\Support\Facades\Http;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('generates a valid strategy from a natural language description', function () {
    $graph = validFormModeGraph();

    Http::fake([
        'api.anthropic.com/*' => Http::response([
            'content' => [['type' => 'text', 'text' => json_encode($graph)]],
        ]),
    ]);

    $this->actingAs($this->user)
        ->postJson(route('strategies.generate'), ['description' => 'Buy UP when price drops more than 5%'])
        ->assertOk()
        ->assertJsonStructure(['graph' => ['mode', 'conditions', 'action', 'risk']]);
});

it('returns 422 when the AI returns invalid JSON', function () {
    Http::fake([
        'api.anthropic.com/*' => Http::response([
            'content' => [['type' => 'text', 'text' => 'not valid json at all']],
        ]),
    ]);

    $this->actingAs($this->user)
        ->postJson(route('strategies.generate'), ['description' => 'Buy UP when price drops more than 5%'])
        ->assertStatus(422)
        ->assertJsonStructure(['error']);
});

it('returns 422 when description is too short', function () {
    $this->actingAs($this->user)
        ->postJson(route('strategies.generate'), ['description' => 'buy'])
        ->assertStatus(422)
        ->assertJsonValidationErrors(['description']);
});

it('returns 422 when the AI returns an invalid graph structure', function () {
    $invalidGraph = ['mode' => 'form', 'conditions' => [], 'action' => [], 'risk' => []];

    Http::fake([
        'api.anthropic.com/*' => Http::response([
            'content' => [['type' => 'text', 'text' => json_encode($invalidGraph)]],
        ]),
    ]);

    $this->actingAs($this->user)
        ->postJson(route('strategies.generate'), ['description' => 'Buy UP when price drops more than 5%'])
        ->assertStatus(422)
        ->assertJsonStructure(['error']);
});

it('handles API errors gracefully', function () {
    Http::fake([
        'api.anthropic.com/*' => Http::response(null, 500),
    ]);

    $this->actingAs($this->user)
        ->postJson(route('strategies.generate'), ['description' => 'Buy UP when price drops more than 5%'])
        ->assertStatus(422)
        ->assertJsonStructure(['error']);
});

it('requires authentication', function () {
    $this->postJson(route('strategies.generate'), ['description' => 'Buy UP when price drops'])
        ->assertUnauthorized();
});

it('strips markdown code fences from AI response', function () {
    $graph = validFormModeGraph();

    Http::fake([
        'api.anthropic.com/*' => Http::response([
            'content' => [['type' => 'text', 'text' => "```json\n".json_encode($graph)."\n```"]],
        ]),
    ]);

    $this->actingAs($this->user)
        ->postJson(route('strategies.generate'), ['description' => 'Buy UP when price drops more than 5%'])
        ->assertOk()
        ->assertJsonPath('graph.mode', 'form');
});

function validFormModeGraph(): array
{
    return [
        'mode' => 'form',
        'conditions' => [
            [
                'type' => 'AND',
                'rules' => [
                    ['indicator' => 'abs_move_pct', 'operator' => '>', 'value' => 5.0],
                    ['indicator' => 'spread_up', 'operator' => '<', 'value' => 0.03],
                ],
            ],
        ],
        'action' => [
            'signal' => 'buy',
            'outcome' => 'UP',
            'size_mode' => 'fixed',
            'size_usdc' => 50,
            'order_type' => 'market',
        ],
        'risk' => [
            'stoploss_pct' => 30,
            'take_profit_pct' => 80,
            'max_position_usdc' => 200,
            'max_trades_per_slot' => 1,
        ],
    ];
}
