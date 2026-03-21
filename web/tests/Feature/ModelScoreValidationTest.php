<?php

use App\Models\User;

beforeEach(function () {
    $this->user = User::factory()->create();
});

function nodeGraphWithModelScore(array $modelScoreNodes): array
{
    $nodes = [];
    foreach ($modelScoreNodes as $i => $data) {
        $nodes[] = [
            'id' => "n{$i}",
            'type' => 'model_score',
            'data' => $data,
        ];
    }

    return [
        'name' => 'Model Strategy',
        'mode' => 'node',
        'graph' => [
            'mode' => 'node',
            'nodes' => $nodes,
            'edges' => [],
        ],
    ];
}

it('accepts valid model_score nodes', function () {
    $payload = nodeGraphWithModelScore([
        ['url' => 'https://ml.example.com/predict', 'json_path' => 'proba_up', 'interval_ms' => 2000],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertRedirect(route('strategies.index'));
});

it('rejects non-HTTPS model_score URLs', function () {
    $payload = nodeGraphWithModelScore([
        ['url' => 'http://ml.example.com/predict', 'json_path' => 'proba_up', 'interval_ms' => 2000],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects private model_score URLs', function () {
    $payload = nodeGraphWithModelScore([
        ['url' => 'https://10.0.0.1/predict', 'json_path' => 'proba_up', 'interval_ms' => 2000],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects empty json_path on model_score nodes', function () {
    $payload = nodeGraphWithModelScore([
        ['url' => 'https://ml.example.com/predict', 'json_path' => '', 'interval_ms' => 2000],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects model_score interval below 1000 milliseconds', function () {
    $payload = nodeGraphWithModelScore([
        ['url' => 'https://ml.example.com/predict', 'json_path' => 'proba_up', 'interval_ms' => 500],
    ]);

    $this->actingAs($this->user)
        ->post(route('strategies.store'), $payload)
        ->assertSessionHasErrors();
});

it('rejects more than 5 model_score nodes', function () {
    $nodes = [];
    for ($i = 0; $i < 6; $i++) {
        $nodes[] = ['url' => "https://ml.example.com/predict/{$i}", 'json_path' => 'proba_up', 'interval_ms' => 2000];
    }

    $this->actingAs($this->user)
        ->post(route('strategies.store'), nodeGraphWithModelScore($nodes))
        ->assertSessionHasErrors();
});
