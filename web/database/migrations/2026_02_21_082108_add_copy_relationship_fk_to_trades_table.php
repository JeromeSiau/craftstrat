<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    /**
     * Run the migrations.
     */
    public function up(): void
    {
        Schema::table('trades', function (Blueprint $table) {
            $table->foreign('copy_relationship_id')
                  ->references('id')
                  ->on('copy_relationships')
                  ->nullOnDelete();
        });
    }

    public function down(): void
    {
        Schema::table('trades', function (Blueprint $table) {
            $table->dropForeign(['copy_relationship_id']);
        });
    }
};
