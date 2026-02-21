<?php

namespace Database\Factories;

use App\Models\Wallet;
use App\Models\WatchedWallet;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\CopyRelationship>
 */
class CopyRelationshipFactory extends Factory
{
    /**
     * Define the model's default state.
     *
     * @return array<string, mixed>
     */
    public function definition(): array
    {
        return [
            'follower_wallet_id' => Wallet::factory(),
            'watched_wallet_id' => WatchedWallet::factory(),
            'size_mode' => fake()->randomElement(['fixed', 'proportional']),
            'size_value' => fake()->randomFloat(6, 10, 500),
            'max_position_usdc' => 100,
            'markets_filter' => null,
            'is_active' => true,
        ];
    }
}
