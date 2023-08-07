using Godot;
using System;

[GlobalClass]
public partial class Entity : RigidBody2D
{
    public enum WishLinearVelocty
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
    public WishLinearVelocty WishLinearVeloctyType = WishLinearVelocty.None;
    [Export]
    public Vector2 WishLinearDirection = Vector2.Zero;


    public enum WishAngularVelocity
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
        /// Set angular velocity to reach a rotation without overshoot.
        /// </summary>
        Rotation,
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
    public WishAngularVelocity WishAngularVelocityType = WishAngularVelocity.None;
    [Export]
    public float WishAngularDirection = 0.0f;

    /// <summary>
    /// Set angular velocity to try to face a point without overshoot.
    /// </summary>
    /// <param name="point">
    /// Point in world space.
    /// </param>
    public void WithAngularVelocityAim(Vector2 point)
    {
        WishAngularVelocityType = WishAngularVelocity.Offset;
        WishAngularDirection = GetAngleToCorrected(point);
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

    public override void _Ready()
    {
        CustomIntegrator = true;
        // Engine.TimeScale = 2.0f;
    }


    public override void _PhysicsProcess(double delta)
    {
    }


    public override void _IntegrateForces(PhysicsDirectBodyState2D state)
    {
        float delta = (float)state.Step * LocalTimeScale;

        // Angular velocity
        float angvel = (float)state.AngularVelocity;
        float newAngvel = angvel;
        float angacc = AngularAcceleration;
        float angvelMax = AngularVelocityMax;
        switch (WishAngularVelocityType)
        {
            case WishAngularVelocity.None:
                break;
            case WishAngularVelocity.Keep:
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
            case WishAngularVelocity.Stop:
                if (!Mathf.IsZeroApprox(angvel))
                {
                    newAngvel = VelocityIntegration.Angvel(
                        0.0f,
                        angvel,
                        angacc,
                        delta
                    );
                }
                break;
            case WishAngularVelocity.Rotation:
                float angvelTarget = Mathf.Deg2Rad(WishAngularDirection);
                float angvelDiff = angvelTarget - angvel;
                if (Mathf.Abs(angvelDiff) < 0.1f)
                {
                    angacc = 0.0f;
                    angvel = angvelTarget;
                }
                else
                {
                    angacc = Mathf.Sign(angvelDiff) * angacc;
                }
                break;
            case WishAngularVelocity.Offset:
                break;
            case WishAngularVelocity.Force:
                angacc = Mathf.Clamp(WishAngularDirection, -1.0f, 1.0f) * angacc;
                break;
        }


    }
}
