CREATE TABLE IF NOT EXISTS simulation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    timestamp DATE NOT NULL
);

CREATE TABLE IF NOT EXISTS environment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simulation_id INTEGER NOT NULL,
    simulation_index INTEGER,
    FOREIGN KEY(simulation_id) REFERENCES simulation(id)
);

CREATE TABLE IF NOT EXISTS agent (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    environment_id INTEGER NOT NULL,
    FOREIGN KEY(environment_id) REFERENCES environment(id)
);

CREATE TABLE IF NOT EXISTS environment_timestep (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    environment_id INTEGER NOT NULL,
    timestep INTEGER,
    FOREIGN KEY(environment_id) REFERENCES environment(id)
);

CREATE TABLE IF NOT EXISTS environment_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simulation_timestep_id INTEGER NOT NULL,
    fires TEXT,
    intensity TEXT,
    suppressants TEXT,
    capacity TEXT,
    equipment TEXT,
    agents TEXT,
    FOREIGN KEY(simulation_timestep_id) REFERENCES environment_timestep(id)
);

CREATE TABLE IF NOT EXISTS agent_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    simulation_timestep_id INTEGER NOT NULL,
    agent_id INTEGER NOT NULL,
    reward INTEGER,
    action_field INTEGER,
    task_field INTEGER,
    action_map TEXT,
    observation_map TEXT,
    FOREIGN KEY(simulation_timestep_id) REFERENCES environment_timestep(id),
    FOREIGN KEY(agent_id) REFERENCES agent(id)
);
