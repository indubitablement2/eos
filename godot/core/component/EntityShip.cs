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
        if (ActionInputs.FaceCursor)
        {
            // Cursor controls.
            Vector2 wishDirection = new Vector2(
                ActionInputs.GetCachedVecticalDirection(),
                ActionInputs.GetCachedHorizontalDirection() + ActionInputs.GetCachedStrafeDirection()
            );

            if (wishDirection.IsZeroApprox())
            {
                if (ActionInputs.CancelLinearVelocity)
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

            SetWishAngularVelocityAim(ActionInputs.GetCachedGlobalMousePosition());
        }
        else
        {
            // Tank controls.
            Vector2 wishDirection = new Vector2(
                ActionInputs.GetCachedVecticalDirection(),
                ActionInputs.GetCachedStrafeDirection()
            );

            if (wishDirection.IsZeroApprox())
            {
                if (ActionInputs.CancelLinearVelocity)
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

            SetWishAngularVelocityWish(ActionInputs.GetCachedHorizontalDirection());
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