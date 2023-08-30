using Godot;

public static class Util
{
    /// <summary>
    /// Normaly an angle of 0.0 points right.
    /// This return the same angle, but 0.0 points up instead.
    /// </summary>
    public static float AngleCorrected(float angle)
    {
        angle += Constant.HalfPi;
        if (angle > Mathf.Pi)
        {
            angle -= Mathf.Tau;
        }

        return angle;
    }
}