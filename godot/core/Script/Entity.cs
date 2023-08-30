using Godot;
using System.Collections.Generic;

[Tool, GlobalClass]
public partial class Entity : RigidBody2D
{
	static (Vector2I, float)[] ARMOR_CELL_EFFECT = new (Vector2I, float)[]
	{
		(new Vector2I(-1, -2), 0.5f),
		(new Vector2I(0, -2), 0.6f),
		(new Vector2I(1, -2), 0.5f),

		(new Vector2I(-2, -1), 0.5f),
		(new Vector2I(-1, -1), 1.0f),
		(new Vector2I(0, -1), 1.0f),
		(new Vector2I(1, -1), 1.0f),
		(new Vector2I(2, -1), 0.5f),

		(new Vector2I(-2, 0), 0.6f),
		(new Vector2I(-1, 0), 1.0f),
		(new Vector2I(0, 0), 1.0f),
		(new Vector2I(1, 0), 1.0f),
		(new Vector2I(2, 0), 0.6f),

		(new Vector2I(-2, 1), 0.5f),
		(new Vector2I(-1, 1), 1.0f),
		(new Vector2I(0, 1), 1.0f),
		(new Vector2I(1, 1), 1.0f),
		(new Vector2I(2, 1), 0.5f),

		(new Vector2I(-1, 2), 0.5f),
		(new Vector2I(0, 2), 0.6f),
		(new Vector2I(1, 2), 0.5f),
	};
	public const float ARMOR_CELL_EFFECT_TOTAL = 15.4f;
	public const float ARMOR_SCALE = 8.0f;
	// TODO: Experiment with size of 3x3
	public static Vector2I ARMOR_SIZE_MIN = new Vector2I(5, 5);
	static Vector2I ARMOR_CENTER_MIN = new Vector2I(2, 2);
	static Vector2I ARMOR_CENTER_MAX = new Vector2I(3, 3);

	const float RECENT_DAMAGE_REMOVE_RATE = 0.25f;
	/// <summary>
	/// How much hull damage / hull max to reach 1.0 recent damage.
	/// </summary>
	const float RECENT_DAMAGE_EFFECT = 0.1f;

	const string HULL_SHADER_PATH = "res://Core/Shader/Hull.gdshader";
	static StringName _armorMaxRelativeTextureName = new StringName("armor_max_texture");
	static StringName _armorRelativeTextureName = new StringName("armor_texture");
	static StringName _recentDamageTextureName = new StringName("recent_damage_texture");

	enum ToolEnum
	{
		None,
		Verify,
		CenterSprite,
	}
	[Export]
	ToolEnum Tool
	{
		get => ToolEnum.None;
		set
		{
			if (!Engine.IsEditorHint()) return;

			switch (value)
			{
				case ToolEnum.None:
					break;
				case ToolEnum.Verify:
					{
						CustomIntegrator = true;
						MaxContactsReported = 4;
						ContactMonitor = true;
						CanSleep = false;

						Data ??= new EntityData();
						Data.Verify();

						if (Data.SimpleArmor)
						{
							if (Material is ShaderMaterial mat)
							{
								if (mat.Shader.ResourcePath == HULL_SHADER_PATH)
								{
									Material = null;
								}
							}
						}
						else
						{
							_recentDamageImage = Image.Create(
								Data.ArmorSize.X, Data.ArmorSize.Y, false, Image.Format.Rf);
							_recentDamageImage.ResourceLocalToScene = true;

							RecentDamageTexture = ImageTexture.CreateFromImage(_recentDamageImage);
							RecentDamageTexture.ResourceLocalToScene = true;

							ArmorRelativeTexture = ImageTexture.CreateFromImage(Data.ArmorMaxRelative);
							ArmorRelativeTexture.ResourceLocalToScene = true;

							Material ??= new ShaderMaterial();
							ShaderMaterial mat = Material as ShaderMaterial;
							mat.Shader = GD.Load<Shader>(HULL_SHADER_PATH);
							Material = mat;
							Material.ResourceLocalToScene = true;

							mat.SetShaderParameter(_armorMaxRelativeTextureName, Data.ArmorMaxRelativeTexture);
							mat.SetShaderParameter(_armorRelativeTextureName, ArmorRelativeTexture);
							mat.SetShaderParameter(_recentDamageTextureName, RecentDamageTexture);
						}

						LinearAcceleration = Data.LinearAcceleration;
						LinearVelocityMax = Data.LinearVelocityMax;
						AngularAcceleration = Data.AngularAcceleration;
						AngularVelocityMax = Data.AngularVelocityMax;
						LocalTimeScale = Data.LocalTimeScale;
						HullMax = Data.HullMax;
						ArmorMax = Data.ArmorMax;
						ArmorMinEffectiveness = Data.ArmorMinEffectiveness;

						ArmorRelative = (Image)Data.ArmorMaxRelative.Duplicate();
						ArmorRelative.ResourceLocalToScene = true;

						for (int i = 0; i < GetChildCount(); i++)
						{
							Node child = GetChild(i);
							if (child is TurretSlot turretSlot)
							{
								turretSlot.Verify();
							}
						}
					}
					break;
				case ToolEnum.CenterSprite:
					Data.SpriteOffset = Data.Sprite.GetSize() * -0.5f;
					break;
			}

			QueueRedraw();
		}
	}

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
		/// Cancel our current velocity to reach position as fast as possible. Does not overshoot.
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
	public WishLinearVeloctyEnum WishLinearVelocityType = WishLinearVeloctyEnum.None;
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
		/// Set angular velocity to reach a rotation offset from current rotation without overshoot.
		/// </summary>
		Offset,
		/// <summary>
		/// Rotate left or right [-1..1]. Force should be clamped to 1.
		/// </summary>
		Force,
	}
	[Export]
	public WishAngularVelocityEnum WishAngularVelocityType = WishAngularVelocityEnum.None;
	[Export]
	public float WishAngularVelocity = 0.0f;

	/// <summary>
	/// Set angular velocity to try to face a point without overshoot.
	/// </summary>
	/// <param name="point">
	/// Point in world space.
	/// </param>
	public void WishAngularVelocityAim(Vector2 point)
	{
		WishAngularVelocityType = WishAngularVelocityEnum.Offset;
		WishAngularVelocity = GetAngleToCorrected(point);
	}

	/// <summary>
	/// Set angular velocity to reach an absolute rotation without overshoot.
	/// </summary>
	public void WishAngularVelocityRotation(float wishRotation)
	{
		WishAngularVelocityType = WishAngularVelocityEnum.Offset;
		WishAngularVelocity = wishRotation - Rotation;
		if (WishAngularVelocity > Mathf.Pi)
		{
			WishAngularVelocity -= Mathf.Tau;
		}
		else if (WishAngularVelocity < -Mathf.Pi)
		{
			WishAngularVelocity += Mathf.Tau;
		}
	}

	[Export]
	public EntityData Data;

	/// <summary>
	/// null if no ai.
	/// </summary>
	[Export]
	public EntityAi Ai;

	// Stats based on data.
	[ExportGroup("Computed")]

	[Export]
	public float LinearAcceleration;
	[Export]
	public float LinearVelocityMax;
	[Export]
	public float AngularAcceleration;
	[Export]
	public float AngularVelocityMax;

	[Export]
	public float LocalTimeScale;

	[Export]
	public float HullMax;
	[Export]
	public float Hull;

	[Export]
	public float ArmorMax;

	[Export]
	public float ArmorMinEffectiveness;

	/// <summary>
	/// How much armor for each cell.
	/// 1.0 == armor_max * ARMOR_CELL_EFFECT_TOTAL
	/// </summary>
	[Export]
	public Image ArmorRelative;
	bool _dirtyArmorRelativeTexture;
	/// <summary>
	/// null if simple_armor
	/// </summary>
	[Export]
	public ImageTexture ArmorRelativeTexture;

	// bool _justTookDamage;

	float _lastRecentDamage;
	/// <summary>
	/// null if simple_armor
	/// </summary>
	// Dictionary<Vector2I, float> _recentDamage;
	HashSet<Vector2I> _recentDamage;
	/// <summary>
	/// null if simple_armor
	/// </summary>
	[Export]
	Image _recentDamageImage;
	bool _dirtyRecentDamageTexture;
	/// <summary>
	/// null if simple_armor
	/// </summary>
	[Export]
	public ImageTexture RecentDamageTexture;

	/// <summary>
	/// null if no target.
	/// </summary>
	public Entity Target;
	public bool AutoTurretDisabled;

	public bool PlayerControlled;
	/// <summary>
	/// What position should manual turrets aim at?
	/// Inf used as a flag for turrets to take their default rotation.
	/// </summary>
	public Vector2 PlayerAimAt = Vector2.Inf;
	/// <summary>
	/// 0: none
	/// 1..14: just pressed actions (respective auto only flags also on)
	/// 14..28 auto only actions
	/// 
	/// To fire all manual turrets, set this to int.MaxValue
	/// </summary>
	public int PlayerActions = 0;

	public override void _Ready()
	{
		Tool = ToolEnum.Verify;

		if (!Data.SimpleArmor)
		{
			_recentDamage = new HashSet<Vector2I>();
		}

		Ai?.Ready(this);
	}

	public override void _Draw()
	{
		DrawTexture(Data.Sprite, Data.SpriteOffset);
	}

	public override void _PhysicsProcess(double delta)
	{
		if (Engine.IsEditorHint()) return;

		float scaledDelta = DeltaScaled((float)delta);

		Ai?.Process(this, scaledDelta);

		if (RecentDamageTexture != null)
		{
			_lastRecentDamage += scaledDelta;

			if (IsInView())
			{
				UpdateRecentDamage();

				if (_dirtyArmorRelativeTexture)
				{
					_dirtyArmorRelativeTexture = false;
					ArmorRelativeTexture.Update(ArmorRelative);
				}

				if (_dirtyRecentDamageTexture)
				{
					_dirtyRecentDamageTexture = false;
					RecentDamageTexture.Update(_recentDamageImage);
				}
			}
		}
	}

	public override void _IntegrateForces(PhysicsDirectBodyState2D state)
	{
		// Contacts
		// TODO: Contact events
		for (int i = 0; i < state.GetContactCount(); ++i)
		{
			Vector2 contactImpulse = state.GetContactImpulse(i);
			float contactLength = contactImpulse.LengthSquared();
			if (contactLength > 400.0f)
			{
				contactLength = Mathf.Sqrt(contactLength);
				Vector2 contactPosition = state.GetContactLocalPosition(i);

				Damage(contactLength, contactPosition, 1.0f);
			}
		}

		float delta = DeltaScaled(state.Step);

		// Angular velocity
		float angvel = (float)state.AngularVelocity;
		float newAngvel = angvel;
		switch (WishAngularVelocityType)
		{
			case WishAngularVelocityEnum.None:
				break;
			case WishAngularVelocityEnum.Keep:
				if (Mathf.Abs(angvel) > AngularVelocityMax)
				{
					newAngvel = VelocityIntegration.Angvel(
						Mathf.Clamp(angvel, AngularVelocityMax, -AngularVelocityMax),
						angvel,
						AngularAcceleration,
						delta
					);
				}
				break;
			case WishAngularVelocityEnum.Stop:
				if (!Mathf.IsZeroApprox(angvel))
				{
					newAngvel = VelocityIntegration.StopAngvel(
						angvel,
						AngularAcceleration,
						delta
					);
				}
				break;
			case WishAngularVelocityEnum.Offset:
				{
					float wishDir = Mathf.Sign(WishAngularVelocity);

					float closeSmooth = Mathf.Min(Mathf.Abs(WishAngularVelocity), 0.2f) / 0.2f;
					closeSmooth *= closeSmooth * closeSmooth;

					if (wishDir == Mathf.Sign(angvel))
					{
						float timeToTarget = Mathf.Abs(WishAngularVelocity / angvel);
						float timeToStop = Mathf.Abs(angvel / AngularAcceleration);

						if (timeToTarget < timeToStop) closeSmooth *= -1.0f;
					}

					newAngvel = VelocityIntegration.Angvel(
						wishDir * AngularVelocityMax * closeSmooth,
						angvel,
						AngularAcceleration,
						delta);
				}
				break;
			case WishAngularVelocityEnum.Force:
				newAngvel = VelocityIntegration.Angvel(
					WishAngularVelocity * AngularVelocityMax,
					angvel,
					AngularAcceleration,
					delta);
				break;
		}

		// Linear velocity
		Vector2 linvel = state.LinearVelocity;
		Vector2 newLinvel = linvel;
		switch (WishLinearVelocityType)
		{
			case WishLinearVeloctyEnum.None:
				break;
			case WishLinearVeloctyEnum.Keep:
				{
					float linvelMaxSquared = LinearVelocityMax * LinearVelocityMax;
					if (linvel.LengthSquared() > linvelMaxSquared)
					{
						newLinvel = VelocityIntegration.StopLinvel(
							linvel,
							LinearAcceleration,
							delta);
						if (newLinvel.LengthSquared() < linvelMaxSquared
							&& !linvel.IsZeroApprox())
						{
							newLinvel = linvel.Normalized() * LinearVelocityMax;
						}
					}
				}
				break;
			case WishLinearVeloctyEnum.Stop:
				newLinvel = VelocityIntegration.StopLinvel(
					linvel,
					LinearAcceleration,
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
							LinearAcceleration,
							delta);
					}
					else
					{
						newLinvel = VelocityIntegration.Linvel(
							target.LimitLength(LinearVelocityMax),
							linvel,
							LinearAcceleration,
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
							new Vector2(0.0f, LinearVelocityMax),
							linvel,
							LinearAcceleration,
							delta);
					}
					else
					{
						newLinvel = VelocityIntegration.Linvel(
							target.Normalized() * LinearVelocityMax,
							linvel,
							LinearAcceleration,
							delta);
					}
				}
				break;
			case WishLinearVeloctyEnum.ForceAbsolute:
				newLinvel = VelocityIntegration.Linvel(
					WishLinearVelocity * LinearVelocityMax,
					linvel,
					LinearAcceleration,
					delta);
				break;
			case WishLinearVeloctyEnum.ForceRelative:
				newLinvel = VelocityIntegration.Linvel(
					WishLinearVelocity.Rotated(Rotation) * LinearVelocityMax,
					linvel,
					LinearAcceleration,
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


	/// <summary>
	/// Normaly an angle of 0.0 points right.
	/// This is the same as GetAngleTo, but 0.0 points up instead.
	/// </summary>
	public float GetAngleToCorrected(Vector2 point)
	{
		return Util.AngleCorrected(GetAngleTo(point));
	}

	public bool IsInView()
	{
		// TODO: Implement this.
		return true;
	}

	public float DeltaScaled(float delta)
	{
		return delta * LocalTimeScale;
	}

	public void UpdateRecentDamage()
	{
		if (_recentDamage == null) return;

		if (_recentDamage.Count == 0 || _lastRecentDamage == 0.0f)
		{
			_lastRecentDamage = 0.0f;
			return;
		}

		_dirtyRecentDamageTexture = true;

		float recentDamageRemove = _lastRecentDamage * RECENT_DAMAGE_REMOVE_RATE;
		_lastRecentDamage = 0.0f;

		List<Vector2I> keysToRemove = new List<Vector2I>();
		foreach (Vector2I point in _recentDamage)
		{
			float newValue = _recentDamageImage.GetPixelv(point).R - recentDamageRemove;
			if (newValue <= 0.0)
			{
				keysToRemove.Add(point);
				_recentDamageImage.SetPixelv(point, new Color(0.0f, 0.0f, 0.0f));
			}
			else
			{
				_recentDamageImage.SetPixelv(point, new Color(newValue, 0.0f, 0.0f));
			}
		}
		// Remove keys with values less than 0.0.
		foreach (Vector2I key in keysToRemove)
		{
			_recentDamage.Remove(key);
		}
	}

	public void Damage(float amount, Vector2 globalPosition, float armorDamageMultiplier)
	{
		DamageLocal(amount, ToLocal(globalPosition), armorDamageMultiplier);
	}

	public void DamageLocal(float amount, Vector2 localPosition, float armorDamageMultiplier)
	{
		const float min_armor = 0.1f;

		// Find nearest armor cell.  
		localPosition -= Data.SpriteOffset;
		localPosition /= ARMOR_SCALE;
		Vector2I centerPoint = (Vector2I)localPosition;
		centerPoint = centerPoint.Clamp(ARMOR_CENTER_MIN, Data.ArmorSize - ARMOR_CENTER_MAX);

		// Fetch total armor.
		float armor = 0.0f;
		foreach ((Vector2I offset, float effect) in ARMOR_CELL_EFFECT)
		{
			armor += ArmorRelative.GetPixelv(centerPoint + offset).R * effect;
		}
		armor /= ARMOR_CELL_EFFECT_TOTAL;
		armor = Mathf.Max(armor, min_armor);

		// Compute damage.
		float dmgReduction = amount / (amount + armor * ArmorMax);
		float hullDamage = amount * dmgReduction;
		float armorDamage = hullDamage / ArmorMax / ARMOR_CELL_EFFECT_TOTAL * armorDamageMultiplier;

		_dirtyArmorRelativeTexture = true;
		foreach ((Vector2I offset, float effect) in ARMOR_CELL_EFFECT)
		{
			Vector2I point = centerPoint + offset;
			float newArmor = ArmorRelative.GetPixelv(point).R - armorDamage * effect;
			newArmor = Mathf.Max(newArmor, 0.0f);
			ArmorRelative.SetPixelv(point, new Color(newArmor, 0.0f, 0.0f));
		}

		if (_recentDamage != null)
		{
			UpdateRecentDamage();
			_dirtyRecentDamageTexture = true;

			float recentDamageAdd = hullDamage / (HullMax * RECENT_DAMAGE_EFFECT);
			foreach ((Vector2I offset, float effect) in ARMOR_CELL_EFFECT)
			{
				Vector2I point = centerPoint + offset;
				_recentDamage.Add(point);
				float newRecentDamage = _recentDamageImage.GetPixelv(point).R + recentDamageAdd * effect;
				newRecentDamage = Mathf.Min(newRecentDamage, 1.0f);
				_recentDamageImage.SetPixelv(point, new Color(newRecentDamage, 0.0f, 0.0f));
			}
		}

		Hull -= hullDamage;
	}
}
