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
        Entity entity = this;
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
                    Vector2 maybe = LinearVelocityIntegration.Stop(linearVelocity, LinearAceleration);
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
                    return LinearVelocityIntegration.Stop(linearVelocity, LinearAceleration);
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
                        return LinearVelocityIntegration.Stop(linearVelocity, LinearAceleration);
                    }
                }
                else
                {
                    return LinearVelocityIntegration.Wish(
                        target.LimitLength(MaxLinearVelocity),
                        linearVelocity,
                        LinearAceleration
                    );
                }
            }
            case WishLinearVelocity.POSITION_OVERSHOT:
            {
                Vector2 target = _wishLinearVelocity - Position;
                return LinearVelocityIntegration.Wish(
                    target.Normalized() * MaxLinearVelocity,
                    linearVelocity,
                    LinearAceleration
                );
            }
            case WishLinearVelocity.ABSOLUTE:
            {
                return LinearVelocityIntegration.Wish(
                    _wishLinearVelocity * MaxLinearVelocity,
                    linearVelocity,
                    LinearAceleration
                );
            }
            case WishLinearVelocity.RELATIVE:
            {
                return LinearVelocityIntegration.Wish(
                    _wishLinearVelocity.Rotated(Rotation) * MaxLinearVelocity,
                    linearVelocity,
                    LinearAceleration
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

    /// <summary>
    /// -1 for no fleet.
    /// </summary>
    public Int64 FleetId = -1;
    public int FleetShipIdx = -1; 

    /// <summary>
    /// -1 for no owner.
    /// </summary>
    public Int64 OwnerClientId = -1;
    public int Team = -1;

    [ExportCategory("Movement")]
    [Export]
    public float BaseLinearAceleration = 500.0f;
    [Export]
    public float BaseAngularAcceleration = 4.0f;
    [Export]
    public float BaseMaxLinearVelocity = 1000.0f;
    [Export]
    public float BaseMaxAngularVelocity = 8.0f;

    public float LinearAceleration;
    public float AngularAcceleration;
    public float MaxLinearVelocity;
    public float MaxAngularVelocity;

    [ExportCategory("Defence")]
    [Export]
    public float BaseReadiness = 500.0f;
    [Export]
    public float BaseHullHp = 1000.0f;
    [Export]
    public float BaseArmorHp = 500.0f;

    public float Readiness;
    public float HullHp;
    public float ArmorHp;

    Entity()
    {
        init();
    }

    Entity
    (
        int team = -1,
        Int64 fleetId = -1,
        int fleetShipIdx = -1,
        Int64 ownerClientId = -1
    )
    {
        Team = team;
        FleetId = fleetId;
        FleetShipIdx = fleetShipIdx;
        OwnerClientId = ownerClientId;

        init();
    }

    void init()
    {
        // TODO: Create a new team. 

        LinearAceleration = BaseLinearAceleration;
        AngularAcceleration = BaseAngularAcceleration;
        MaxLinearVelocity = BaseMaxLinearVelocity;
        MaxAngularVelocity = BaseMaxAngularVelocity;

        Readiness = BaseReadiness;
        HullHp = BaseHullHp;
        ArmorHp = BaseArmorHp;
    }

    public override void _PhysicsProcess(double delta)
    {
        
    }

    public override void _IntegrateForces(PhysicsDirectBodyState2D state)
    {
        state.AngularVelocity = IntegrateAngularVelocity(state.AngularVelocity);
        state.LinearVelocity = IntegrateLinearVelocity(state.LinearVelocity);
    }
}
