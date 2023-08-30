using Godot;

[Tool, GlobalClass]
public partial class TurretSlot : Node2D
{
	public enum TurretWeightEnum
	{
		Light,
		Meidum,
		Heavy,
	}

	[Export]
	public TurretWeightEnum TurretWeightMax = TurretWeightEnum.Light;

	/// <summary>
	///  If > PI, can rotate without blocking.
	/// </summary>
	[Export(PropertyHint.Range, "0, 3.1416, 0.1")]
	public float FiringArc
	{
		get => _firingArc; set
		{
			_firingArc = value;
			if (Engine.IsEditorHint()) QueueRedraw();
		}
	}
	float _firingArc = 3.1416f;

	// [ExportGroup("Computed")]
	// /// <summary>
	// /// The base angle offset from the front the entity needs for this turret to be able to fire.
	// ///	</summary>
	// [Export]
	// public float BaseFiringWishOffset;

	public override void _Draw()
	{
		if (Engine.IsEditorHint() && FiringArc < Mathf.Pi)
		{
			DrawLine(
				Vector2.Zero,
				new Vector2(0.0f, -100.0f).Rotated(-FiringArc),
				Colors.AliceBlue,
				-1.0f,
				true);
			DrawLine(
				Vector2.Zero,
				new Vector2(0.0f, -100.0f).Rotated(FiringArc),
				Colors.AliceBlue,
				-1.0f,
				true);
		}
	}

	public void Verify()
	{
	}
}
