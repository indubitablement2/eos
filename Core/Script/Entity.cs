using Godot;
using System;


[GlobalClass]
public partial class Entity : RigidBody2D
{
    public enum WishLinearVeloctyEnum
    {
        /// <summary>
        /// Keep current linear velocity.
        /// </summary>
        None,
        /// <summary>
        /// Keep current linear velocity.
        /// Do nothing unless above max, then slow down until back to max.
        /// </summary>
        Keep,
        /// <summary>
        /// Try to reach 0 linear velocity.
        /// </summary>
        Stop,
        /// <summary>
        /// Cancel our current velocity to reach position as fast as possible.
        /// Does not overshoot.
        /// </summary>
        PositionSmooth,
        /// <summary>
        /// Same as PositionSmooth, but always try to go at max velocity.
        /// </summary>
        PositionOvershoot,
        /// <summary>
        /// Force toward an absolute direction. -y is up.
        /// Magnitude bellow 1 can be used to accelerate slower.
        /// Magnitude should be clamped to 1.
        /// </summary>
        ForceAbsolute,
        /// <summary>
        /// Force toward a direction relative to current rotation. -y is forward.
        /// Magnitude bellow 1 can be used to accelerate slower.
        /// Magnitude should be clamped to 1.
        /// </summary>
        ForceRelative,
    }
    [Export]
    public WishLinearVeloctyEnum WishLinearVeloctyType = WishLinearVeloctyEnum.None;
    [Export]
    public Vector2 WishLinearVelocity = Vector2.Zero;


    public enum WishAngularVelocityEnum
    {
        /// <summary>
        /// Keep current angular velocity.
        /// </summary>
        None,
        /// <summary>
        /// Keep current angular velocity.
        /// Do nothing unless above max, then slow down until back to max.
        /// </summary>
        Keep,
        /// <summary>
        /// Try to reach 0 angular velocity.
        /// </summary>
        Stop,
        /// <summary>
        /// Set angular velocity to reach a rotation offset from current rotation
        /// without overshoot.
        /// </summary>
        Offset,
        /// <summary>
        /// Rotate left or right [-1..1].
	    /// Force should be clamped to 1.
        /// </summary>
        Force,
    }
    [Export]
    public WishAngularVelocityEnum WishAngularVelocityType = WishAngularVelocityEnum.None;
    [Export]
    public float WishAngularDirection = 0.0f;

    /// <summary>
    /// Set angular velocity to try to face a point without overshoot.
    /// </summary>
    /// <param name="point">
    /// Point in world space.
    /// </param>
    public void WishAngularVelocityAim(Vector2 point)
    {
        WishAngularVelocityType = WishAngularVelocityEnum.Offset;
        WishAngularDirection = GetAngleToCorrected(point);
    }

    /// <summary>
    /// Set angular velocity to reach an absolute rotation without overshoot.
    /// </summary>
    public void WishAngularVelocityRotation(float wishRotation)
    {
        WishAngularVelocityType = WishAngularVelocityEnum.Offset;
        WishAngularDirection = wishRotation - Rotation;
        if (WishAngularDirection > Mathf.Pi)
        {
            WishAngularDirection -= Mathf.Tau;
        }
        else if (WishAngularDirection < -Mathf.Pi)
        {
            WishAngularDirection += Mathf.Tau;
        }
    }


    [Export(PropertyHint.Range, "0, 2000, 1, or_greater")]
    public float LinearAcceleration = 800.0f;
    [Export(PropertyHint.Range, "0, 2000, 1, or_greater")]
    public float LinearVelocityMax = 400.0f;
    [Export(PropertyHint.Range, "0, 100, 1, or_greater")]
    public float AngularAcceleration = 8.0f;
    [Export(PropertyHint.Range, "0, 100, 1, or_greater")]
    public float AngularVelocityMax = 4.0f;


    [Export(PropertyHint.Range, "0.01, 4, 0.01, or_greater")]
    public float LocalTimeScale = 1.0f;

    public float EffectiveDelta(float delta)
    {
        return delta * LocalTimeScale;
    }


    // [ExportGroup("Save")]
    // [Export]
    // public float Ok;
    // [Export]
    // public float OkK;


    public Entity Target = null;
    /// <summary>
    /// Inf used as a flag for turrets to take their default rotation.
    /// </summary>
    public Vector2 AimAt = Vector2.Inf;
    /// <summary>
    /// 0: none
    /// 1..14: just pressed actions (respective auto only flags also on)
    /// 14..28 auto only actions
    /// </summary>
    public int Actions = 0;


    /// <summary>
    /// Normaly an angle of 0.0 points right.
    /// This is the same as GetAngleTo, but 0.0 is up.
    /// </summary>
    public float GetAngleToCorrected(Vector2 point)
    {
        float angle = GetAngleTo(point);
        angle += Util.HalfPi;
        if (angle > Mathf.Pi)
        {
            angle -= Mathf.Tau;
        }

        return angle;
    }


    public Entity()
    {
        CenterOfMassMode = CenterOfMassModeEnum.Custom;
        CenterOfMass = Vector2.Zero;

        CustomIntegrator = true;
        MaxContactsReported = 4;
        ContactMonitor = true;
        CanSleep = false;
    }


    public override void _Ready()
    {
        // Engine.TimeScale = 2.0f;
    }


    public override void _PhysicsProcess(double delta)
    {
    }


    public override void _IntegrateForces(PhysicsDirectBodyState2D state)
    {
        float delta = EffectiveDelta(state.Step);

        // Angular velocity
        float angvel = (float)state.AngularVelocity;
        float newAngvel = angvel;
        float angacc = AngularAcceleration;
        float angvelMax = AngularVelocityMax;
        switch (WishAngularVelocityType)
        {
            case WishAngularVelocityEnum.None:
                break;
            case WishAngularVelocityEnum.Keep:
                if (Mathf.Abs(angvel) > angvelMax)
                {
                    newAngvel = VelocityIntegration.Angvel(
                        Mathf.Clamp(angvel, angvelMax, -angvelMax),
                        angvel,
                        angacc,
                        delta
                    );
                }
                break;
            case WishAngularVelocityEnum.Stop:
                if (!Mathf.IsZeroApprox(angvel))
                {
                    newAngvel = VelocityIntegration.StopAngvel(
                        angvel,
                        angacc,
                        delta
                    );
                }
                break;
            case WishAngularVelocityEnum.Offset:
                {
                    float wishDir = Mathf.Sign(WishAngularDirection);

                    float closeSmooth = Mathf.Min(Mathf.Abs(WishAngularDirection), 0.2f) / 0.2f;
                    closeSmooth *= closeSmooth * closeSmooth;

                    if (wishDir == Mathf.Sign(angvel))
                    {
                        float timeToTarget = Mathf.Abs(WishAngularDirection / angvel);
                        float timeToStop = Mathf.Abs(angvel / angacc);

                        if (timeToTarget < timeToStop) closeSmooth *= -1.0f;
                    }

                    newAngvel = VelocityIntegration.Angvel(
                        wishDir * angvelMax * closeSmooth,
                        angvel,
                        angacc,
                        delta);
                }
                break;
            case WishAngularVelocityEnum.Force:
                newAngvel = VelocityIntegration.Angvel(
                    WishAngularDirection * angvelMax,
                    angvel,
                    angacc,
                    delta);
                break;
        }

        // Linear velocity
        Vector2 linvel = state.LinearVelocity;
        Vector2 newLinvel = linvel;
        float linacc = LinearAcceleration;
        float linvelMax = LinearVelocityMax;
        switch (WishLinearVeloctyType)
        {
            case WishLinearVeloctyEnum.None:
                break;
            case WishLinearVeloctyEnum.Keep:
                {
                    float linvelMaxSquared = linvelMax * linvelMax;
                    if (linvel.LengthSquared() > linvelMaxSquared)
                    {
                        newLinvel = VelocityIntegration.StopLinvel(
                            linvel,
                            linacc,
                            delta);
                        if (newLinvel.LengthSquared() < linvelMaxSquared
                            && !linvel.IsZeroApprox())
                        {
                            newLinvel = linvel.Normalized() * linvelMax;
                        }
                    }
                }
                break;
            case WishLinearVeloctyEnum.Stop:
                newLinvel = VelocityIntegration.StopLinvel(
                    linvel,
                    linacc,
                    delta);
                break;
            case WishLinearVeloctyEnum.PositionSmooth:
                {
                    Vector2 target = WishLinearVelocity - Position;
                    if (target.LengthSquared() < 100.0f)
                    {
                        // We are alreay on target.
                        newLinvel = VelocityIntegration.StopLinvel(
                            linvel,
                            linacc,
                            delta);
                    }
                    else
                    {
                        newLinvel = VelocityIntegration.Linvel(
                            target.LimitLength(linvelMax),
                            linvel,
                            linacc,
                            delta);
                    }
                }
                break;
            case WishLinearVeloctyEnum.PositionOvershoot:
                {
                    Vector2 target = WishLinearVelocity - Position;
                    if (target.LengthSquared() < 1.0f)
                    {
                        newLinvel = VelocityIntegration.Linvel(
                            new Vector2(0.0f, linvelMax),
                            linvel,
                            linacc,
                            delta);
                    }
                    else
                    {
                        newLinvel = VelocityIntegration.Linvel(
                            target.Normalized() * linvelMax,
                            linvel,
                            linacc,
                            delta);
                    }
                }
                break;
            case WishLinearVeloctyEnum.ForceAbsolute:
                newLinvel = VelocityIntegration.Linvel(
                    WishLinearVelocity * linvelMax,
                    linvel,
                    linacc,
                    delta);
                break;
            case WishLinearVeloctyEnum.ForceRelative:
                newLinvel = VelocityIntegration.Linvel(
                    WishLinearVelocity.Rotated(Rotation) * linvelMax,
                    linvel,
                    linacc,
                    delta);
                break;
        }

        if (newAngvel != angvel)
        {
            state.AngularVelocity = newAngvel;
        }
        if (newLinvel != linvel)
        {
            state.LinearVelocity = newLinvel;
        }
    }
}
