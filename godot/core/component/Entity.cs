using Godot;
using System;

public partial class Entity : RigidBody2D
{
    enum WishLinearVelocity
    {
        KEEP,
        CANCEL,
        POSITION,
        POSITION_OVERSHOT,
        ABSOLUTE,
        RELATIVE,
    };

    WishLinearVelocity _wishLinearVelocityType = WishLinearVelocity.KEEP;
    Vector2 _wishLinearVelocity = Vector2.Zero;

    /// <summary>
    /// Keep current linear velocity. eg. do nothing.
    /// </summary> 
    public void SetWishLinearVelocityKeep()
    {
        _wishLinearVelocityType = WishLinearVelocity.KEEP;
    }

    /// <summary>
    /// Try to reach 0 linear velocity.
    /// </summary>
    public void SetWishLinearVelocityCancel()
    {
        _wishLinearVelocityType = WishLinearVelocity.CANCEL;
    }

    /// <summary>
    /// Cancel our current velocity to reach position as fast as possible.
    /// Does not overshot.
    /// </summary>
    public void SetWishLinearVelocityPosition(Vector2 position)
    {
        _wishLinearVelocityType = WishLinearVelocity.POSITION;
        _wishLinearVelocity = position;
    }

    /// <summary>
    /// Same as position, but always try to go at max velocity and will overshot.
    /// </summary>
    public void SetWishLinearVelocityPositionOvershot(Vector2 position)
    {
        _wishLinearVelocityType = WishLinearVelocity.POSITION_OVERSHOT;
        _wishLinearVelocity = position;
    }

    /// <summary>
    /// Force toward an absolute direction. -y is up. 
    /// Magnitude bellow 1 can be used to accelerate slower.
    /// Magnitude clamped to 1.
    /// </summary>
    public void SetWishLinearVelocityAbsolute(Vector2 direction)
    {
        _wishLinearVelocityType = WishLinearVelocity.ABSOLUTE;
        _wishLinearVelocity = direction.LimitLength(1.0f);
    }

    /// <summary>
    /// Force toward a direction relative to current rotation. +y is forward. 
    /// Magnitude bellow 1 can be used to accelerate slower.
    /// Magnitude clamped to 1.
    /// </summary>
    public void SetWishLinearVelocityRelative(Vector2 direction)
    {
        _wishLinearVelocityType = WishLinearVelocity.RELATIVE;
        _wishLinearVelocity = direction.LimitLength(1.0f);
    }

    Vector2 IntegrateLinearVelocity(Vector2 linearVelocity)
    {
        switch (_wishLinearVelocityType)
        {
            case WishLinearVelocity.KEEP:
                {
                    float maxLinearVelocitySquared = Stats.MaxLinearVelocity * Stats.MaxLinearVelocity;
                    if (linearVelocity.LengthSquared() > maxLinearVelocitySquared)
                    {
                        // Slow down to max velocity.
                        Vector2 maybe = LinearVelocityIntegration.Stop(linearVelocity, Stats.LinearAceleration);
                        if (maybe.LengthSquared() < maxLinearVelocitySquared)
                        {
                            // Slowed down too much, set to max velocity instead.
                            return linearVelocity.Normalized() * Stats.MaxLinearVelocity;
                        }
                        return maybe;
                    }
                    else
                    {
                        return linearVelocity;
                    }
                }
            case WishLinearVelocity.CANCEL:
                {
                    if (linearVelocity.LengthSquared() < 1.0f)
                    {
                        return Vector2.Zero;
                    }
                    else
                    {
                        return LinearVelocityIntegration.Stop(linearVelocity, Stats.LinearAceleration);
                    }
                }
            case WishLinearVelocity.POSITION:
                {
                    Vector2 target = _wishLinearVelocity - Position;
                    if (target.LengthSquared() < 10.0f)
                    {
                        // We are alreay on target.
                        if (linearVelocity.LengthSquared() < 1.0f)
                        {
                            return Vector2.Zero;
                        }
                        else
                        {
                            return LinearVelocityIntegration.Stop(linearVelocity, Stats.LinearAceleration);
                        }
                    }
                    else
                    {
                        return LinearVelocityIntegration.Wish(
                            target.LimitLength(Stats.MaxLinearVelocity),
                            linearVelocity,
                            Stats.LinearAceleration
                        );
                    }
                }
            case WishLinearVelocity.POSITION_OVERSHOT:
                {
                    Vector2 target = _wishLinearVelocity - Position;
                    return LinearVelocityIntegration.Wish(
                        target.Normalized() * Stats.MaxLinearVelocity,
                        linearVelocity,
                        Stats.LinearAceleration
                    );
                }
            case WishLinearVelocity.ABSOLUTE:
                {
                    return LinearVelocityIntegration.Wish(
                        _wishLinearVelocity * Stats.MaxLinearVelocity,
                        linearVelocity,
                        Stats.LinearAceleration
                    );
                }
            case WishLinearVelocity.RELATIVE:
                {
                    return LinearVelocityIntegration.Wish(
                        _wishLinearVelocity.Rotated(Rotation) * Stats.MaxLinearVelocity,
                        linearVelocity,
                        Stats.LinearAceleration
                    );
                }
            default:
                {
                    return linearVelocity;
                }
        }
    }

    enum WishAngularVelocity
    {
        KEEP,
        CANCEL,
        AIM,
        FORCE,
    };

    WishAngularVelocity _wishAngularVelocityType = WishAngularVelocity.KEEP;
    Vector2 _wishAngularVelocity = Vector2.Zero;

    /// <summary>
    /// Keep current angular velocity. eg. do nothing.
    /// </summary>
    public void SetWishAngularVelocityKeep()
    {
        _wishAngularVelocityType = WishAngularVelocity.KEEP;
    }

    /// <summary>
    /// Try to reach 0 angular velocity.
    /// </summary>
    public void SetWishAngularVelocityCancel()
    {
        _wishAngularVelocityType = WishAngularVelocity.CANCEL;
    }

    /// <summary>
    /// Set angular velocity to face world space position without overshot.
    /// </summary>
    public void SetWishAngularVelocityAim(Vector2 position)
    {
        _wishAngularVelocityType = WishAngularVelocity.AIM;
        _wishAngularVelocity = position;
    }

    /// <summary>
    /// Rotate left or right [-1..1].
    /// Force will be clamped.
    /// </summary>
    public void SetWishAngularVelocityForce(float force)
    {
        _wishAngularVelocityType = WishAngularVelocity.FORCE;
        _wishAngularVelocity.X = Math.Clamp(force, -1.0f, 1.0f);
    }

    float IntegrateAngularVelocity(float angularVelocity)
    {
        switch (_wishAngularVelocityType)
        {
            case WishAngularVelocity.KEEP:
                if (angularVelocity > Stats.MaxAngularVelocity)
                {
                    return Math.Max(
                        AngularVelocityIntegration.Stop(angularVelocity, Stats.AngularAcceleration),
                        Stats.MaxAngularVelocity
                    );
                }
                else if (angularVelocity < -Stats.MaxAngularVelocity)
                {
                    return Math.Min(
                        AngularVelocityIntegration.Stop(angularVelocity, Stats.AngularAcceleration),
                        Stats.MaxAngularVelocity
                    );
                }
                else
                {
                    return angularVelocity;
                }
            case WishAngularVelocity.CANCEL:
                return AngularVelocityIntegration.Stop(
                    angularVelocity,
                    Stats.AngularAcceleration
                );
            case WishAngularVelocity.AIM:
                return AngularVelocityIntegration.Offset(
                    GetAngleTo(_wishAngularVelocity),
                    angularVelocity,
                    Stats.AngularAcceleration,
                    Stats.MaxAngularVelocity
                );
            case WishAngularVelocity.FORCE:
                return AngularVelocityIntegration.Force(
                    _wishAngularVelocity.X,
                    angularVelocity,
                    Stats.AngularAcceleration,
                    Stats.MaxAngularVelocity
                );
            default:
                return angularVelocity;
        }
    }

    public EntityData Data;
    public EntityStats Stats;

    public Entity(EntityStats stats, EntityData data)
    {
        Data = data;
        Stats = stats;
    }

    public override void _IntegrateForces(PhysicsDirectBodyState2D state)
    {
        state.AngularVelocity = IntegrateAngularVelocity(state.AngularVelocity);
        state.LinearVelocity = IntegrateLinearVelocity(state.LinearVelocity);
    }
}
