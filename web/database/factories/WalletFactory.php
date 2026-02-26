<?php

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

/**
 * @extends \Illuminate\Database\Eloquent\Factories\Factory<\App\Models\Wallet>
 */
class WalletFactory extends Factory
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
            'label' => fake()->optional()->words(2, true),
            'signer_address' => '0x'.fake()->regexify('[a-fA-F0-9]{40}'),
            'safe_address' => '0x'.fake()->regexify('[a-fA-F0-9]{40}'),
            'private_key_enc' => base64_encode(fake()->sha256()),
            'status' => 'deployed',
            'balance_usdc' => fake()->randomFloat(6, 0, 10000),
            'is_active' => true,
            'deployed_at' => now(),
        ];
    }

    /**
     * Indicate that the wallet is inactive.
     */
    public function inactive(): static
    {
        return $this->state(fn () => ['is_active' => false]);
    }

    /**
     * Indicate that the wallet is pending Safe deployment.
     */
    public function pending(): static
    {
        return $this->state(fn () => [
            'safe_address' => null,
            'status' => 'pending',
            'deployed_at' => null,
        ]);
    }

    /**
     * Indicate that the wallet Safe is currently deploying.
     */
    public function deploying(): static
    {
        return $this->state(fn () => [
            'safe_address' => null,
            'status' => 'deploying',
            'deployed_at' => null,
        ]);
    }

    /**
     * Indicate that the wallet Safe deployment failed.
     */
    public function failed(): static
    {
        return $this->state(fn () => [
            'safe_address' => null,
            'status' => 'failed',
            'deployed_at' => null,
        ]);
    }
}
