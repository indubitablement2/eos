using Godot;
using System;

public partial class EntityShip : Entity
{
    public Int64 OwnerClientId = -1;

    /// <summary>
    /// Tick beyond which the ship is allowed to leave the battlescape.
    /// </summary>
    public Int64 LeaveTick;

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
                Input.GetActionStrength(Actions.Up) - Input.GetActionStrength(Actions.Down),
                Input.GetActionStrength(Actions.Right) - Input.GetActionStrength(Actions.Left) + Input.GetActionStrength(Actions.StrafeRight) - Input.GetActionStrength(Actions.StrafeLeft)

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
                Input.GetActionStrength(Actions.Up) - Input.GetActionStrength(Actions.Down),
                Input.GetActionStrength(Actions.StrafeRight) - Input.GetActionStrength(Actions.StrafeLeft)

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
        base._PhysicsProcess(delta);
        Controlled();

        // if (OwnerClientId == Metascape.LocalClientId)
        // {

        // }
        // else
        // {
        //     Ai();
        // }
    }
}