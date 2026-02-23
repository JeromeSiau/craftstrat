<?php

namespace App\Services;

use App\Exceptions\StrategyGenerationException;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Str;

class StrategyGeneratorService
{
    private const VALID_INDICATORS = [
        'abs_move_pct', 'dir_move_pct', 'spread_up', 'spread_down',
        'size_ratio_up', 'size_ratio_down', 'pct_into_slot', 'minutes_into_slot',
        'mid_up', 'mid_down', 'bid_up', 'ask_up', 'bid_down', 'ask_down',
        'ref_price', 'hour_utc', 'day_of_week', 'market_volume_usd',
    ];

    private const VALID_OPERATORS = ['>', '<', '>=', '<=', '==', '!=', 'between'];

    public function __construct(
        private string $apiKey,
        private string $model,
    ) {}

    /**
     * Generate a FormModeGraph from a natural language description.
     *
     * @return array{graph: array<string, mixed>}
     */
    public function generate(string $description): array
    {
        $response = Http::withHeaders([
            'x-api-key' => $this->apiKey,
            'anthropic-version' => '2023-06-01',
        ])
            ->timeout(30)
            ->post('https://api.anthropic.com/v1/messages', [
                'model' => $this->model,
                'max_tokens' => 1024,
                'system' => $this->systemPrompt(),
                'messages' => [
                    ['role' => 'user', 'content' => $description],
                ],
            ]);

        if (! $response->successful()) {
            throw StrategyGenerationException::apiError($response->status());
        }

        $raw = $response->json('content.0.text', '');
        $graph = $this->parseAndValidate($raw);

        return ['graph' => $graph];
    }

    /**
     * Parse the LLM response and validate it as a FormModeGraph.
     *
     * @return array<string, mixed>
     */
    public function parseAndValidate(string $raw): array
    {
        $raw = $this->stripCodeFences($raw);

        $graph = json_decode($raw, true);
        if (! is_array($graph)) {
            throw StrategyGenerationException::invalidJson($raw);
        }

        $this->validate($graph);
        $this->ensureIds($graph);

        return $graph;
    }

    /**
     * @param  array<string, mixed>  $graph
     */
    private function validate(array $graph): void
    {
        if (($graph['mode'] ?? null) !== 'form') {
            throw StrategyGenerationException::validationFailed('mode must be "form"');
        }

        $conditions = $graph['conditions'] ?? null;
        if (! is_array($conditions) || $conditions === []) {
            throw StrategyGenerationException::validationFailed('conditions must be a non-empty array');
        }

        foreach ($conditions as $i => $group) {
            $this->validateConditionGroup($group, $i);
        }

        $this->validateAction($graph['action'] ?? null);
        $this->validateRisk($graph['risk'] ?? null);
    }

    private function validateConditionGroup(mixed $group, int $index): void
    {
        if (! is_array($group)) {
            throw StrategyGenerationException::validationFailed("conditions[{$index}] must be an object");
        }

        if (! in_array($group['type'] ?? null, ['AND', 'OR'], true)) {
            throw StrategyGenerationException::validationFailed("conditions[{$index}].type must be AND or OR");
        }

        $rules = $group['rules'] ?? null;
        if (! is_array($rules) || $rules === []) {
            throw StrategyGenerationException::validationFailed("conditions[{$index}].rules must be non-empty");
        }

        foreach ($rules as $j => $rule) {
            $this->validateRule($rule, $index, $j);
        }
    }

    private function validateRule(mixed $rule, int $groupIndex, int $ruleIndex): void
    {
        $path = "conditions[{$groupIndex}].rules[{$ruleIndex}]";

        if (! is_array($rule)) {
            throw StrategyGenerationException::validationFailed("{$path} must be an object");
        }

        if (! in_array($rule['indicator'] ?? null, self::VALID_INDICATORS, true)) {
            throw StrategyGenerationException::validationFailed("{$path}.indicator is invalid");
        }

        if (! in_array($rule['operator'] ?? null, self::VALID_OPERATORS, true)) {
            throw StrategyGenerationException::validationFailed("{$path}.operator is invalid");
        }

        $value = $rule['value'] ?? null;
        if ($rule['operator'] === 'between') {
            if (! is_array($value) || count($value) !== 2 || ! is_numeric($value[0]) || ! is_numeric($value[1])) {
                throw StrategyGenerationException::validationFailed("{$path}.value must be [min, max] for between");
            }
        } elseif (! is_numeric($value)) {
            throw StrategyGenerationException::validationFailed("{$path}.value must be numeric");
        }
    }

    private function validateAction(mixed $action): void
    {
        if (! is_array($action)) {
            throw StrategyGenerationException::validationFailed('action must be an object');
        }

        if (! in_array($action['signal'] ?? null, ['buy', 'sell'], true)) {
            throw StrategyGenerationException::validationFailed('action.signal must be buy or sell');
        }

        if (! in_array($action['outcome'] ?? null, ['UP', 'DOWN'], true)) {
            throw StrategyGenerationException::validationFailed('action.outcome must be UP or DOWN');
        }

        if (! in_array($action['size_mode'] ?? null, ['fixed', 'proportional'], true)) {
            throw StrategyGenerationException::validationFailed('action.size_mode must be fixed or proportional');
        }

        if (! is_numeric($action['size_usdc'] ?? null) || $action['size_usdc'] < 1) {
            throw StrategyGenerationException::validationFailed('action.size_usdc must be >= 1');
        }

        if (! in_array($action['order_type'] ?? null, ['market', 'limit'], true)) {
            throw StrategyGenerationException::validationFailed('action.order_type must be market or limit');
        }
    }

    private function validateRisk(mixed $risk): void
    {
        if (! is_array($risk)) {
            throw StrategyGenerationException::validationFailed('risk must be an object');
        }

        if (! is_numeric($risk['max_position_usdc'] ?? null) || $risk['max_position_usdc'] < 1) {
            throw StrategyGenerationException::validationFailed('risk.max_position_usdc must be >= 1');
        }

        if (! is_numeric($risk['max_trades_per_slot'] ?? null) || $risk['max_trades_per_slot'] < 1) {
            throw StrategyGenerationException::validationFailed('risk.max_trades_per_slot must be >= 1');
        }

        if (isset($risk['stoploss_pct']) && $risk['stoploss_pct'] !== null && (! is_numeric($risk['stoploss_pct']) || $risk['stoploss_pct'] <= 0)) {
            throw StrategyGenerationException::validationFailed('risk.stoploss_pct must be > 0 or null');
        }

        if (isset($risk['take_profit_pct']) && $risk['take_profit_pct'] !== null && (! is_numeric($risk['take_profit_pct']) || $risk['take_profit_pct'] <= 0)) {
            throw StrategyGenerationException::validationFailed('risk.take_profit_pct must be > 0 or null');
        }

        if (isset($risk['daily_loss_limit_usdc']) && $risk['daily_loss_limit_usdc'] !== null && (! is_numeric($risk['daily_loss_limit_usdc']) || $risk['daily_loss_limit_usdc'] <= 0)) {
            throw StrategyGenerationException::validationFailed('risk.daily_loss_limit_usdc must be > 0 or null');
        }

        if (isset($risk['cooldown_seconds']) && $risk['cooldown_seconds'] !== null && (! is_numeric($risk['cooldown_seconds']) || $risk['cooldown_seconds'] <= 0)) {
            throw StrategyGenerationException::validationFailed('risk.cooldown_seconds must be > 0 or null');
        }
    }

    /**
     * Ensure all condition groups and rules have unique IDs.
     *
     * @param  array<string, mixed>  $graph
     */
    private function ensureIds(array &$graph): void
    {
        foreach ($graph['conditions'] as &$group) {
            if (empty($group['id'])) {
                $group['id'] = (string) Str::uuid();
            }
            foreach ($group['rules'] as &$rule) {
                if (empty($rule['id'])) {
                    $rule['id'] = (string) Str::uuid();
                }
            }
        }
    }

    private function stripCodeFences(string $raw): string
    {
        $raw = trim($raw);

        if (str_starts_with($raw, '```')) {
            $raw = preg_replace('/^```(?:json)?\s*/', '', $raw);
            $raw = preg_replace('/\s*```$/', '', $raw);
        }

        return trim($raw);
    }

    private function systemPrompt(): string
    {
        return <<<'PROMPT'
You are a trading strategy generator for Polymarket prediction markets.
You output ONLY valid JSON matching the FormModeGraph schema below. No markdown, no explanation, no commentary.

## Schema

{
  "mode": "form",
  "conditions": [
    {
      "type": "AND" or "OR",
      "rules": [
        {
          "indicator": "<indicator_name>",
          "operator": "<operator>",
          "value": <number> or [<min>, <max>] for "between"
        }
      ]
    }
  ],
  "action": {
    "signal": "buy" or "sell",
    "outcome": "UP" or "DOWN",
    "size_mode": "fixed",
    "size_usdc": <number >= 1>,
    "order_type": "market" or "limit"
  },
  "risk": {
    "stoploss_pct": <number > 0> or null,
    "take_profit_pct": <number > 0> or null,
    "max_position_usdc": <number >= 1>,
    "max_trades_per_slot": <number >= 1>
  }
}

## Available Indicators

Price:
- abs_move_pct: Absolute price movement % since slot start (typically 0-15)
- dir_move_pct: Directional (signed) price movement % since slot start (-15 to 15)
- mid_up: Midpoint price for UP outcome (0 to 1)
- mid_down: Midpoint price for DOWN outcome (0 to 1)
- ref_price: Reference price from Chainlink oracle

Spread:
- spread_up: Bid-ask spread for UP outcome (0 to 0.1, lower = tighter)
- spread_down: Bid-ask spread for DOWN outcome (0 to 0.1)

Order Book:
- size_ratio_up: Bid/ask size ratio for UP (>1 = more buyers, <1 = more sellers)
- size_ratio_down: Bid/ask size ratio for DOWN
- bid_up: Best bid for UP (0 to 1)
- ask_up: Best ask for UP (0 to 1)
- bid_down: Best bid for DOWN (0 to 1)
- ask_down: Best ask for DOWN (0 to 1)

Time:
- pct_into_slot: % of time elapsed in current slot (0 to 1)
- minutes_into_slot: Minutes elapsed in current slot
- hour_utc: Current hour in UTC (0 to 23)
- day_of_week: Day of week (0=Sunday to 6=Saturday)

Volume:
- market_volume_usd: Total market volume in USD for current slot

## Available Operators
>, <, >=, <=, ==, !=, between

## Examples

User: "Buy UP when price drops more than 5% and spread is tight"
{
  "mode": "form",
  "conditions": [{"type": "AND", "rules": [
    {"indicator": "abs_move_pct", "operator": ">", "value": 5.0},
    {"indicator": "spread_up", "operator": "<", "value": 0.03}
  ]}],
  "action": {"signal": "buy", "outcome": "UP", "size_mode": "fixed", "size_usdc": 50, "order_type": "market"},
  "risk": {"stoploss_pct": 30, "take_profit_pct": 80, "max_position_usdc": 200, "max_trades_per_slot": 1}
}

User: "Sell DOWN at end of slot when book is imbalanced toward sellers"
{
  "mode": "form",
  "conditions": [{"type": "AND", "rules": [
    {"indicator": "pct_into_slot", "operator": ">", "value": 0.8},
    {"indicator": "size_ratio_down", "operator": "<", "value": 0.5}
  ]}],
  "action": {"signal": "sell", "outcome": "DOWN", "size_mode": "fixed", "size_usdc": 30, "order_type": "market"},
  "risk": {"stoploss_pct": 25, "take_profit_pct": 60, "max_position_usdc": 150, "max_trades_per_slot": 1}
}

User: "Buy UP during weekday mornings when volume is high and price is cheap"
{
  "mode": "form",
  "conditions": [{"type": "AND", "rules": [
    {"indicator": "day_of_week", "operator": "between", "value": [1, 5]},
    {"indicator": "hour_utc", "operator": "between", "value": [8, 14]},
    {"indicator": "market_volume_usd", "operator": ">", "value": 10000},
    {"indicator": "mid_up", "operator": "<", "value": 0.4}
  ]}],
  "action": {"signal": "buy", "outcome": "UP", "size_mode": "fixed", "size_usdc": 100, "order_type": "market"},
  "risk": {"stoploss_pct": 20, "take_profit_pct": 50, "max_position_usdc": 300, "max_trades_per_slot": 2}
}

## Rules
- Always use realistic, conservative risk parameters.
- Always include stoploss_pct and take_profit_pct (non-null).
- Use "fixed" for size_mode unless the user specifically asks for proportional.
- Use "market" for order_type unless the user specifically asks for limit orders.
- Output ONLY the JSON object. No other text.
PROMPT;
    }
}
