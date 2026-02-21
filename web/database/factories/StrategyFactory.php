<?php

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Strategy>
 */
class StrategyFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'name' => fake()->words(3, true),
            'description' => fake()->optional()->sentence(),
            'graph' => [
                'mode' => 'form',
                'conditions' => [
                    [
                        'type' => 'AND',
                        'rules' => [
                            ['indicator' => 'abs_move_pct', 'operator' => '>', 'value' => 3.0],
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
            ],
            'mode' => 'form',
            'is_active' => false,
        ];
    }

    /**
     * Indicate that the strategy is active.
     */
    public function active(): static
    {
        return $this->state(fn () => ['is_active' => true]);
    }

    /**
     * Indicate that the strategy uses node mode.
     */
    public function nodeMode(): static
    {
        return $this->state(fn () => [
            'mode' => 'node',
            'graph' => [
                'mode' => 'node',
                'nodes' => [
                    ['id' => 'n1', 'type' => 'input', 'data' => ['field' => 'abs_move_pct']],
                    ['id' => 'n2', 'type' => 'comparator', 'data' => ['operator' => '>', 'value' => 3.0]],
                    ['id' => 'n3', 'type' => 'action', 'data' => ['signal' => 'buy', 'outcome' => 'UP', 'size_usdc' => 50]],
                ],
                'edges' => [
                    ['source' => 'n1', 'target' => 'n2'],
                    ['source' => 'n2', 'target' => 'n3'],
                ],
            ],
        ]);
    }
}
