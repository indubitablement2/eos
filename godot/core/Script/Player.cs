using Godot;
using System;

public partial class Player : Node2D
{
	static Entity _controlled;
	public static Entity Controlled
	{
		set
		{
			if (_controlled != null)
			{
				_controlled.TreeExiting -= () => _controlled = null;
				_controlled.PlayerControlled = false;
			}

			_controlled = value;
			if (_controlled != null)
			{
				_controlled.TreeExiting += () => _controlled = null;
				_controlled.PlayerControlled = true;
			}
		}
		get => _controlled;
	}

	public static Vector2 MousePosition;

	public override void _Ready()
	{
	}

	public override void _Process(double delta)
	{
		MousePosition = GetGlobalMousePosition();

		if (Controlled != null)
		{
			Controlled.PlayerAimAt = MousePosition;

			Vector2 wishDir = new Vector2(
				Input.GetActionStrength(Constant.InputRight)
					- Input.GetActionStrength(Constant.InputLeft),
				Input.GetActionStrength(Constant.InputDown)
					- Input.GetActionStrength(Constant.InputUp)
			);
			if (Input.IsActionPressed(Constant.InputAimAtCursor))
			{
				// Drone control.
				Controlled.WishAngularVelocityAim(MousePosition);
				Controlled.WishLinearVelocityType = Entity.WishLinearVeloctyEnum.ForceAbsolute;
				Controlled.WishLinearVelocity = wishDir.LimitLength(1.0f);
			}
			else
			{
				Controlled.WishAngularVelocityType = Entity.WishAngularVelocityEnum.Force;
				Controlled.WishAngularVelocity = wishDir.X;
				Controlled.WishLinearVelocityType = Entity.WishLinearVeloctyEnum.ForceRelative;
				// TODO: Strafe
				Controlled.WishLinearVelocity = new Vector2(0.0f, wishDir.Y);
			}

			Controlled.PlayerActions = 0;
			Controlled.PlayerActions |= Input.IsActionPressed(Constant.InputPrimary) ? 1 : 0;
			Controlled.PlayerActions |= Input.IsActionPressed(Constant.InputSecondary) ? 2 : 0;
		}
	}
}
