using Godot;

public static class VelocityIntegration
{
    /// <summary>
    /// Return the linear velocity after applying a force to stop.
    /// </summary>
    public static Vector2 StopLinvel(Vector2 linvel, float linacc, float delta)
    {
        return linvel - linvel.LimitLength(linacc * delta);
    }

    /// <summary>
    /// Return the linear velocity after applying a force to reach wishLinvel.
    /// Does not care about max velocity.
    /// wishLinvel should already be capped.
    /// </summary>
    public static Vector2 Linvel(Vector2 wishLinvel, Vector2 linvel, float linacc, float delta)
    {
        return linvel + (wishLinvel - linvel).LimitLength(linacc * delta);
    }

    /// <summary>
    // Return the angular velocity after applying a force to stop.
    /// </summary>
    public static float StopAngvel(float angvel, float angacc, float delta)
    {
        return angvel - Mathf.Clamp(angvel, -angacc * delta, angacc * delta);
    }

    /// <summary>
    /// Return the angular velocity after applying a force to reach wishAngvel.
    /// Does not care about max velocity.
    /// wishAngvel should already be capped.
    /// </summary>
    public static float Angvel(float wishAngvel, float angvel, float angacc, float delta)
    {
        return angvel + Mathf.Clamp(wishAngvel - angvel, -angacc * delta, angacc * delta);
    }

}

