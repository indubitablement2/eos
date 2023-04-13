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
                    float maxLinearVelocitySquared = MaxLinearVelocity * MaxLinearVelocity;
                    if (linearVelocity.LengthSquared() > maxLinearVelocitySquared)
                    {
                        // Slow down to max velocity.
                        Vector2 maybe = LinearVelocityIntegration.Stop(linearVelocity, LinearAcceleration);
                        if (maybe.LengthSquared() < maxLinearVelocitySquared)
                        {
                            // Slowed down too much, set to max velocity instead.
                            return linearVelocity.Normalized() * MaxLinearVelocity;
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
                        return LinearVelocityIntegration.Stop(linearVelocity, LinearAcceleration);
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
                            return LinearVelocityIntegration.Stop(linearVelocity, LinearAcceleration);
                        }
                    }
                    else
                    {
                        return LinearVelocityIntegration.Wish(
                            target.LimitLength(MaxLinearVelocity),
                            linearVelocity,
                            LinearAcceleration
                        );
                    }
                }
            case WishLinearVelocity.POSITION_OVERSHOT:
                {
                    Vector2 target = _wishLinearVelocity - Position;
                    return LinearVelocityIntegration.Wish(
                        target.Normalized() * MaxLinearVelocity,
                        linearVelocity,
                        LinearAcceleration
                    );
                }
            case WishLinearVelocity.ABSOLUTE:
                {
                    return LinearVelocityIntegration.Wish(
                        _wishLinearVelocity * MaxLinearVelocity,
                        linearVelocity,
                        LinearAcceleration
                    );
                }
            case WishLinearVelocity.RELATIVE:
                {
                    return LinearVelocityIntegration.Wish(
                        _wishLinearVelocity.Rotated(Rotation) * MaxLinearVelocity,
                        linearVelocity,
                        LinearAcceleration
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
                if (angularVelocity > MaxAngularVelocity)
                {
                    return Math.Max(
                        AngularVelocityIntegration.Stop(angularVelocity, AngularAcceleration),
                        MaxAngularVelocity
                    );
                }
                else if (angularVelocity < -MaxAngularVelocity)
                {
                    return Math.Min(
                        AngularVelocityIntegration.Stop(angularVelocity, AngularAcceleration),
                        MaxAngularVelocity
                    );
                }
                else
                {
                    return angularVelocity;
                }
            case WishAngularVelocity.CANCEL:
                return AngularVelocityIntegration.Stop(
                    angularVelocity,
                    AngularAcceleration
                );
            case WishAngularVelocity.AIM:
                return AngularVelocityIntegration.Offset(
                    GetAngleTo(_wishAngularVelocity),
                    angularVelocity,
                    AngularAcceleration,
                    MaxAngularVelocity
                );
            case WishAngularVelocity.FORCE:
                return AngularVelocityIntegration.Force(
                    _wishAngularVelocity.X,
                    angularVelocity,
                    AngularAcceleration,
                    MaxAngularVelocity
                );
            default:
                return angularVelocity;
        }
    }

    [Export]
    public float Readiness = 500.0f;
    [Export]
    public float HullHp = 1000.0f;
    [Export]
    public float ArmorHp = 500.0f;

    [Export]
    public float LinearAcceleration = 500.0f;
    [Export]
    public float AngularAcceleration = 6.0f;
    [Export]
    public float MaxLinearVelocity = 500.0f;
    [Export]
    public float MaxAngularVelocity = 6.0f;

    public event Action OnDestroyed;

    public Entity()
    {
        SetArmorHp(ArmorHp);
    }

    public void SetArmorHp(float armorHp)
    {
        // TODO: Armor grid.
    }

    public float GetAverageArmorHp()
    {
        return ArmorHp;
    }

    public override void _PhysicsProcess(double delta)
    {
        Readiness -= Constants.Delta;

        if (HullHp < 0.0f)
        {
            QueueFree();
            GD.Print("Destroyed");
            OnDestroyed?.Invoke();
        }
    }

    public override void _IntegrateForces(PhysicsDirectBodyState2D state)
    {
        state.AngularVelocity = IntegrateAngularVelocity(state.AngularVelocity);
        state.LinearVelocity = IntegrateLinearVelocity(state.LinearVelocity);
    }
}
