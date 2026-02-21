<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('strategies', function (Blueprint $table) {
            $table->id();
            $table->foreignId('user_id')->constrained()->cascadeOnDelete();
            $table->string('name');
            $table->text('description')->nullable();
            $table->jsonb('graph');
            $table->string('mode', 10)->default('form');
            $table->boolean('is_active')->default(false);
            $table->timestamps();
        });

        if (DB::getDriverName() === 'pgsql') {
            DB::statement('CREATE INDEX idx_strategies_graph ON strategies USING GIN (graph)');
        }
    }

    public function down(): void
    {
        Schema::dropIfExists('strategies');
    }
};
