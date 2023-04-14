using Godot;
using System;

// TODO: Wrapper around steamworks
public class Client
{
    public Int64 ClientId;

    public Client(Int64 clientId)
    {
        ClientId = clientId;
    }

    public bool IsLocal()
    {
        return true;
    }

    public static Client Default()
    {
        return new Client(-1);
    }
}