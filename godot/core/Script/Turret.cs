using Godot;

[Tool, GlobalClass]
public partial class Turret : Sprite2D
{
	[Export]
	public TurretData Data;

	[ExportGroup("Computed")]
	[Export]
	public float RotationSpeed;
	[Export]
	public float FireDelay;

	[Export]
	public int AmmoMax;
	[Export]
	public float AmmoReplenishDelay;

	[Export]
	public int AmmoReplenishAmount;

	// TODO: Range increase

	[ExportGroup("Save")]
	[Export]
	public int Ammo;
	/// <summary>
	/// Current delay before ammo replenish.
	/// </summary>
	[Export]
	public float AmmoReplenishCooldown;
	[Export]
	public float FireCooldown;

	Entity _target;
	/// <summary>
	/// null if no target.
	/// </summary>
	public Entity Target
	{
		set
		{
			if (_target != null)
			{
				_target.TreeExiting -= () => _target = null;
			}

			_target = value;
			if (_target != null)
			{
				_target.TreeExiting += () => _target = null;
			}
		}
		get => _target;
	}
	Vector2 TargetPosition;

	public TurretSlot TurretSlot;
	public Entity Entity;

	public int ActionGroup;
	// TODO: stored on the area's shape's filter.
	// smalls
	// smalls -> target -> ships
	// target -> ships
	// public bool PointDefence;

	public virtual void Fire()
	{
		return;
	}

	public override void _EnterTree()
	{
		// Turret should always be a child of Entity/TurretSlot.
		TurretSlot = GetParent<TurretSlot>();
		Entity = TurretSlot.GetParent<Entity>();

		AmmoReplenishCooldown = AmmoReplenishDelay;
		Ammo = AmmoMax;
	}

	public override void _ExitTree()
	{
		Target = null;
	}

	public override void _Process(double delta)
	{
		if (Engine.IsEditorHint()) return;

		float scaledDelta = Entity.DeltaScaled((float)delta);

		bool isPlayerControlled = Entity.PlayerControlled && ActionGroup != 0;

		// Find where to aim at.
		Vector2 aimAt = Vector2.Inf;
		bool wishFire = false;
		if (isPlayerControlled)
		{
			aimAt = Entity.PlayerAimAt;
			wishFire = ActionGroup == Entity.PlayerActions;
		}
		else if (!Entity.AutoTurretDisabled)
		{
			if (Target != null)
			{
				// TODO: Check if can reach target
				aimAt = Target.Position;
			}
			else
			{
				if (Entity.Target != null)
				{
					// TODO: Check if can reach entity's target
					if (true)
					{
						Target = Entity.Target;
					}
					else
					{
						// TODO: Query to find a new target.
					}
				}
			}
		}

		// Rotate toward aim at.
		float rotation = Rotation;
		float wishAngleChange = -rotation;
		if (aimAt.X != Mathf.Inf)
		{
			float angleToTarget = Util.AngleCorrected(GetAngleTo(aimAt));
			wishAngleChange = angleToTarget;
			if (isPlayerControlled)
			{
				wishFire = Mathf.Abs(angleToTarget) <= Data.EffectiveAngle;
			}
		}
		float rotationSpeed = RotationSpeed * scaledDelta;
		if (TurretSlot.FiringArc < Mathf.Pi)
		{
			if (Mathf.Abs(rotation + wishAngleChange) > Mathf.Pi)
			{
				wishAngleChange -= Mathf.Sign(wishAngleChange) * Mathf.Tau;
			}

			rotation += Mathf.Clamp(wishAngleChange, -rotationSpeed, rotationSpeed);
			rotation = Mathf.Clamp(rotation, -TurretSlot.FiringArc, TurretSlot.FiringArc);
		}
		else
		{
			rotation += Mathf.Clamp(wishAngleChange, -rotationSpeed, rotationSpeed);
		}
		Rotation = rotation;

		// Replenish ammo.
		if (Ammo < AmmoMax)
		{
			AmmoReplenishCooldown -= scaledDelta;
			if (AmmoReplenishCooldown < 0.0f)
			{
				AmmoReplenishCooldown += AmmoReplenishDelay;
				Ammo = Mathf.Min(Ammo + AmmoReplenishAmount, AmmoMax);
			}
		}
		else
		{
			AmmoReplenishCooldown = AmmoReplenishDelay;
		}

		// Fire.
		FireCooldown -= scaledDelta;
		if (wishFire)
		{
			// TODO: If player controlled, make a sound when out of ammo and trying to fire.

			while (Ammo > 0 && FireCooldown < 0.0f)
			{
				Fire();
				Ammo -= 1;
			}
		}
		FireCooldown = Mathf.Max(FireCooldown, 0.0f);
	}

	/// <summary>
	/// Return the most forward angle offset this turret wish the entity to rotate to.
	/// </summary>
	public float WishAngle()
	{
		float wishAngle = 0.0f;
		float turretSlotRotation = TurretSlot.Rotation;
		float firingArc = TurretSlot.FiringArc + Data.EffectiveAngle;
		if (Mathf.Abs(turretSlotRotation) > firingArc)
		{
			wishAngle = -turretSlotRotation + firingArc * Mathf.Sign(turretSlotRotation);
		}

		return wishAngle;
	}

	public void Verify()
	{
		Data.Verify();
		RotationSpeed = Data.RotationSpeed;
		FireDelay = Data.FireDelay;
		AmmoMax = Data.AmmoMax;
		AmmoReplenishDelay = Data.AmmoReplenishDelay;
		AmmoReplenishAmount = Data.AmmoReplenishAmount;
	}
}

