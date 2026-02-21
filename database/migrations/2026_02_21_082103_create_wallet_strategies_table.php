<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('wallet_strategies', function (Blueprint $table) {
            $table->id();
            $table->foreignId('wallet_id')->constrained()->cascadeOnDelete();
            $table->foreignId('strategy_id')->constrained()->cascadeOnDelete();
            $table->jsonb('markets')->default('[]');
            $table->decimal('max_position_usdc', 18, 6)->default(100);
            $table->boolean('is_running')->default(false);
            $table->timestamp('started_at')->nullable();
            $table->unique(['wallet_id', 'strategy_id']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('wallet_strategies');
    }
};
