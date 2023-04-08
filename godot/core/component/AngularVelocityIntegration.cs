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
    /// Return the angular velocity after applying a force.
    /// force is assumed to be in the range [-1, 1].
    /// </summary>
    public static float Force
    (
        float force,
        float angularVelocity,
        float angularAcceleration,
        float maxAngularVelocity
    )
    {
        if (angularVelocity > maxAngularVelocity)
        {
            if (Math.Sign(force) == Math.Sign(angularVelocity))
            {
                // Trying to go in the same dir as current velocity while speed is over max.
                // Ignore force, slow down to max speed instead.
                return Math.Max(angularVelocity - angularAcceleration * Constants.Delta, maxAngularVelocity);
            }
            else
            {
                // Trying to go in the opposite dir as current velocity while speed is over max.
                float maybe = angularVelocity + force * angularAcceleration * Constants.Delta;
                if (maybe > maxAngularVelocity)
                {
                    // Ignore force, slow down as much as possible to reach max speed instead.
                    return Math.Max(angularVelocity - angularAcceleration * Constants.Delta, maxAngularVelocity);
                }
                else
                {
                    // Force is enough to slow down to max speed.
                    return Math.Max(maybe, -maxAngularVelocity);
                }
            }
        } 
        else if (angularVelocity < -maxAngularVelocity) 
        {
            if (Math.Sign(force) == Math.Sign(angularVelocity)) 
            {
                // Trying to go in the same dir as current velocity while speed is over max.
                // Ignore force, slow down to max speed instead.
                return Math.Min(angularVelocity + angularAcceleration * Constants.Delta, -maxAngularVelocity);
            }
            else
            {
                // Trying to go in the opposite dir as current velocity while speed is over max.
                float maybe = angularVelocity + force * angularAcceleration * Constants.Delta;
                if (maybe < -maxAngularVelocity)
                {
                    // Ignore force, slow down as much as possible to reach max speed instead.
                    return Math.Min(angularVelocity + angularAcceleration  * Constants.Delta, -maxAngularVelocity);
                }
                else
                {
                    // Force is enough to slow down to max speed.
                    return Math.Min(maybe, maxAngularVelocity);
                }
            }
        }
        else 
        {
            // Speed is under max.
            return Math.Clamp(angularVelocity + force * angularAcceleration * Constants.Delta, -maxAngularVelocity, maxAngularVelocity);
        }
    }

    /// <summary>
    /// Return the angular velocity after applying a force to rotate by offset radian.
    /// </summary>
    public static float Offset
    (
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
                return Force((float)Math.Sign(offset), angularVelocity, angularAcceleration, maxAngularVelocity);
            }
        }
        else
        {
            // We are going in the opposite direction, so we can go at full speed.
            return Force((float)Math.Sign(offset), angularVelocity, angularAcceleration, maxAngularVelocity);
        }
    }
}
