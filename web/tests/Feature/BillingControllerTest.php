<?php

use App\Models\User;
use Inertia\Testing\AssertableInertia as Assert;

beforeEach(function () {
    $this->withoutVite();
    $this->user = User::factory()->create();
});

it('displays billing page', function () {
    $this->actingAs($this->user)
        ->get(route('billing.index'))
        ->assertOk()
        ->assertInertia(fn (Assert $page) => $page
            ->component('billing/index', false)
            ->has('plan')
            ->has('subscribed')
        );
});

it('validates price_id on subscribe', function () {
    $this->actingAs($this->user)
        ->post(route('billing.subscribe'), [])
        ->assertSessionHasErrors(['price_id']);
});

it('requires authentication for billing pages', function () {
    $this->get(route('billing.index'))->assertRedirect('/login');
});
