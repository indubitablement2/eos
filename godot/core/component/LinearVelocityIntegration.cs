using Godot;
using System;

public static class LinearVelocityIntegration
{
    // How much to increase acceleration force when stopping.
    const float StopAccelerationMultiplier = 1.05f;

    /// <summary>
    /// Return the linear velocity after applying a force to stop.
    /// </summary>
    public static Vector2 Stop(Vector2 linearVelocity, float linearAcceleration)
    {
        return linearVelocity - linearVelocity.LimitLength(linearAcceleration * StopAccelerationMultiplier * Constants.Delta);
    }

    /// <summary>
    /// Return the linear velocity after applying a force to reach wishLinearVelocity.
    /// Does not care about max velocity.
    /// wishLinearVelocity should already be capped.
    /// </summary>
    public static Vector2 Wish(Vector2 wishLinearVelocity, Vector2 linearVelocity, float linearAcceleration)
    {
        return linearVelocity + (wishLinearVelocity - linearVelocity).LimitLength(linearAcceleration * Constants.Delta);
    }
}
