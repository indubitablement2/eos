using Godot;
using System;

[GlobalClass]
public partial class TestMe : Node
{
    [Export(PropertyHint.Range, "-360,360")]
    public int Banana = 0;

    public string MyName()
    {
        string name = "TestMe";
        GD.Print(name);
        return name;
    }
}
