using Godot;
using System;

public partial class EntityShip : Entity
{
    public Ship Ship;

    /// <summary>
    /// Tick beyond which the ship is allowed to leave the battlescape.
    /// </summary>
    public Int64 LeaveTick;

    public void Initialize(Ship ship, BattlescapeSimulation battlescapeSimulation)
    {
        Ship = ship;
        Initialize(
            Ship.Readiness,
            Ship.HullHp,
            Ship.ArmorHp,
            Ship.ShipData.EntityData,
            battlescapeSimulation
        );
    }

    // ~EntityShip()
    // {
    // TODO: Check if destroyed.
    // TODO: Update ship readiness, hull hp, armor hp.
    //     Ship.State = Ship.ShipState.Ready;
    // }

    void Controlled()
    {
        if (Input.IsActionPressed(Actions.FaceCursor))
        {
            // Cursor controls.
            Vector2 wishDirection = new Vector2(
                Input.GetActionStrength(Actions.Right) - Input.GetActionStrength(Actions.Left) + Input.GetActionStrength(Actions.StrafeRight) - Input.GetActionStrength(Actions.StrafeLeft),
                Input.GetActionStrength(Actions.Down) - Input.GetActionStrength(Actions.Up)
            );

            if (wishDirection.IsZeroApprox())
            {
                if (Input.IsActionPressed(Actions.CancelLinearVelocity))
                {
                    SetWishLinearVelocityCancel();
                }
                else
                {
                    SetWishLinearVelocityKeep();
                }
            }
            else
            {
                SetWishLinearVelocityRelative(wishDirection);
            }

            SetWishAngularVelocityAim(GetGlobalMousePosition());
        }
        else
        {
            // Tank controls.
            Vector2 wishDirection = new Vector2(
                Input.GetActionStrength(Actions.StrafeRight) - Input.GetActionStrength(Actions.StrafeLeft),
                Input.GetActionStrength(Actions.Down) - Input.GetActionStrength(Actions.Up)
            );

            if (wishDirection.IsZeroApprox())
            {
                if (Input.IsActionPressed(Actions.CancelLinearVelocity))
                {
                    SetWishLinearVelocityCancel();
                }
                else
                {
                    SetWishLinearVelocityKeep();
                }
            }
            else
            {
                SetWishLinearVelocityRelative(wishDirection);
            }

            SetWishAngularVelocityForce(
                Input.GetActionStrength(Actions.Right) - Input.GetActionStrength(Actions.Left)
            );
        }
    }

    void Ai()
    {

    }

    public override void _PhysicsProcess(double delta)
    {
        if (Ship.Fleet.OwnerClient != null)
        {
            if (Ship.Fleet.OwnerClient.ClientId == Metascape.LocalClientId)
            {
                Controlled();
            }
            else
            {
                Ai();
            }
        }
        else
        {
            Ai();
        }
    }
}