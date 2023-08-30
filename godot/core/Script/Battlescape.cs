using Godot;

public partial class Battlescape : Node2D
{
    public static Battlescape Instance;

    public override void _EnterTree()
    {
        Instance = this;

        var ship = GD.Load<PackedScene>("res://Base/Ship/Janitor/Janitor.tscn");

        var instance = ship.Instantiate<Entity>();
        instance.Position = new Vector2(200.0f, 500.0f);
        AddChild(instance);
        Player.Controlled = instance;

        for (int i = 0; i < 10; i++)
        {
            instance = ship.Instantiate<Entity>();
            instance.Position = new Vector2(200.0f + i * 100.0f, 500.0f);
            AddChild(instance);
        }
    }

    public override void _ExitTree()
    {
        if (Instance == this) Instance = null;
    }

    public override void _Process(double delta)
    {

    }
}
