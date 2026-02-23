<?php

use App\Exceptions\StrategyGenerationException;
use App\Services\StrategyGeneratorService;

function makeService(): StrategyGeneratorService
{
    return new StrategyGeneratorService(apiKey: 'test-key', model: 'test-model');
}

it('parses a valid JSON response into a FormModeGraph', function () {
    $json = json_encode([
        'mode' => 'form',
        'conditions' => [
            ['type' => 'AND', 'rules' => [['indicator' => 'abs_move_pct', 'operator' => '>', 'value' => 5]]],
        ],
        'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 50, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 30, 'take_profit_pct' => 80, 'max_position_usdc' => 200, 'max_trades_per_slot' => 1],
    ]);

    $graph = makeService()->parseAndValidate($json);

    expect($graph['mode'])->toBe('form')
        ->and($graph['conditions'])->toHaveCount(1)
        ->and($graph['conditions'][0]['rules'][0]['indicator'])->toBe('abs_move_pct');
});

it('strips markdown code fences from the response', function () {
    $raw = "```json\n".json_encode([
        'mode' => 'form',
        'conditions' => [['type' => 'AND', 'rules' => [['indicator' => 'mid_up', 'operator' => '<', 'value' => 0.4]]]],
        'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 25, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 20, 'take_profit_pct' => 60, 'max_position_usdc' => 100, 'max_trades_per_slot' => 1],
    ])."\n```";

    $graph = makeService()->parseAndValidate($raw);

    expect($graph['mode'])->toBe('form');
});

it('throws on invalid JSON', function () {
    makeService()->parseAndValidate('this is not json');
})->throws(StrategyGenerationException::class, 'invalid JSON');

it('throws when indicator is invalid', function () {
    $json = json_encode([
        'mode' => 'form',
        'conditions' => [['type' => 'AND', 'rules' => [['indicator' => 'fake_indicator', 'operator' => '>', 'value' => 1]]]],
        'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 50, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 30, 'take_profit_pct' => 80, 'max_position_usdc' => 200, 'max_trades_per_slot' => 1],
    ]);

    makeService()->parseAndValidate($json);
})->throws(StrategyGenerationException::class, 'indicator is invalid');

it('throws when conditions array is empty', function () {
    $json = json_encode([
        'mode' => 'form',
        'conditions' => [],
        'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 50, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 30, 'take_profit_pct' => 80, 'max_position_usdc' => 200, 'max_trades_per_slot' => 1],
    ]);

    makeService()->parseAndValidate($json);
})->throws(StrategyGenerationException::class, 'conditions must be a non-empty array');

it('throws when action signal is invalid', function () {
    $json = json_encode([
        'mode' => 'form',
        'conditions' => [['type' => 'AND', 'rules' => [['indicator' => 'mid_up', 'operator' => '>', 'value' => 0.5]]]],
        'action' => ['signal' => 'hold', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 50, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 30, 'take_profit_pct' => 80, 'max_position_usdc' => 200, 'max_trades_per_slot' => 1],
    ]);

    makeService()->parseAndValidate($json);
})->throws(StrategyGenerationException::class, 'action.signal must be buy or sell');

it('adds UUIDs to conditions and rules missing IDs', function () {
    $json = json_encode([
        'mode' => 'form',
        'conditions' => [['type' => 'AND', 'rules' => [['indicator' => 'abs_move_pct', 'operator' => '>', 'value' => 3]]]],
        'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 50, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 30, 'take_profit_pct' => 80, 'max_position_usdc' => 200, 'max_trades_per_slot' => 1],
    ]);

    $graph = makeService()->parseAndValidate($json);

    expect($graph['conditions'][0]['id'])->toBeString()->not->toBeEmpty()
        ->and($graph['conditions'][0]['rules'][0]['id'])->toBeString()->not->toBeEmpty();
});

it('validates between operator requires array value', function () {
    $json = json_encode([
        'mode' => 'form',
        'conditions' => [['type' => 'AND', 'rules' => [['indicator' => 'hour_utc', 'operator' => 'between', 'value' => 5]]]],
        'action' => ['signal' => 'buy', 'outcome' => 'UP', 'size_mode' => 'fixed', 'size_usdc' => 50, 'order_type' => 'market'],
        'risk' => ['stoploss_pct' => 30, 'take_profit_pct' => 80, 'max_position_usdc' => 200, 'max_trades_per_slot' => 1],
    ]);

    makeService()->parseAndValidate($json);
})->throws(StrategyGenerationException::class, 'must be [min, max] for between');
