using Godot;
using System;

public static class AngularVelocityIntegration
{
    // How much to increase acceleration force when stopping.
    const float StopAccelerationMultiplier = 1.05f;

    /// <summary>
    /// Return the angular velocity after applying a force to stop.
    /// </summary>
    public static float Stop(float angularVelocity, float angularAcceleration)
    {
        return angularVelocity - Math.Clamp(
            angularVelocity,
            -angularAcceleration * StopAccelerationMultiplier * Constants.Delta,
            angularAcceleration * StopAccelerationMultiplier * Constants.Delta
        );
    }

    /// <summary>
    /// Return the angular velocity after applying a force to rotate by offset radian.
    /// </summary>
    public static float Offset(
        float offset,
        float angularVelocity,
        float angularAcceleration,
        float maxAngularVelocity
    )
    {
        if (Math.Abs(offset) < 0.01f)
        {
            return Stop(angularVelocity, angularAcceleration);
        }
        else if (Math.Sign(offset) == Math.Sign(angularVelocity))
        {
            // Calculate the time to reach 0 angular velocity.
            float timeToStop = Math.Abs(angularVelocity / angularAcceleration);

            // Calculate the time to reach the target.
            float timeToTarget = Math.Abs(offset / angularVelocity);

            if (timeToTarget < timeToStop)
            {
                // We will overshoot the target, so we need to slow down.
                return Stop(angularVelocity, angularAcceleration);
            }
            else
            {
                // We can go at full speed.
                return Wish(Math.Sign(offset) * maxAngularVelocity, angularVelocity, angularAcceleration);
                // return Force((float)Math.Sign(offset), angularVelocity, angularAcceleration, maxAngularVelocity);
            }
        }
        else
        {
            // We are going in the opposite direction, so we can go at full speed.
            return Wish(Math.Sign(offset) * maxAngularVelocity, angularVelocity, angularAcceleration);
        }
    }

    /// <summary>
    /// Return the angular velocity after applying a force reach wishAngularVolicty.
    /// force is assumed to be clamped to MaxAngularVelocity.
    /// </summary>
    public static float Wish(
        float wishAngularVolicty,
        float angularVelocity,
        float angularAcceleration
    )
    {
        return angularVelocity + Math.Clamp(
            wishAngularVolicty - angularVelocity,
            -angularAcceleration * Constants.Delta,
            angularAcceleration * Constants.Delta
        );
    }
}
