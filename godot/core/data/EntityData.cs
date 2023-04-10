using Godot;
using System;

public class EntityData
{
    public string Id;

    public EntityStats BaseStats;

    public EntityData(Resource entityDataResource)
    {
        BaseStats = new EntityStats();

        Id = (string)entityDataResource.Get("id");

        BaseStats.LinearAceleration = (float)entityDataResource.Get("LinearAceleration");
        BaseStats.AngularAcceleration = (float)entityDataResource.Get("AngularAcceleration");
        BaseStats.MaxLinearVelocity = (float)entityDataResource.Get("MaxLinearVelocity");
        BaseStats.MaxAngularVelocity = (float)entityDataResource.Get("MaxAngularVelocity");
        BaseStats.Readiness = (float)entityDataResource.Get("Readiness");
        BaseStats.HullHp = (float)entityDataResource.Get("HullHp");
        BaseStats.ArmorHp = (float)entityDataResource.Get("ArmorHp");
    }
}