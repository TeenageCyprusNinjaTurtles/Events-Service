import faker
import requests
import psycopg2
import json

faker_instance = faker.Faker()
con = psycopg2.connect("postgresql://postgres:cHt0UFBbszX0YK7@localhost:5432")


def test_event_create():
    cur = con.cursor()
    cur.execute("DELETE FROM platform_events")
    con.commit()
    event_obj = {
        "name": faker_instance.company(),
        "location": faker_instance.address(),
        "start": faker_instance.date_time().strftime("%Y-%m-%d %H:%M:%S"),
        "duration": "some_days",
        "description": faker_instance.text(),
    }
    response = requests.post("http://localhost:5001/event/create", json=event_obj)
    assert response.json() == {'result': 'ACCESS_ERROR'}

    response = requests.post("http://localhost:5001/event/create", json=event_obj, headers={"X-User-Email":"admin@mail.com", "X-User-Level":"1"})
    assert response.json() == {'result': 'ACCESS_ERROR'}

    response = requests.post("http://localhost:5001/event/create", json=event_obj, headers={"X-User-Email":"admin@mail.com", "X-User-Level":"2"})
    assert type(response.json()['id'] == int)


    event_obj.pop("duration")
    response = requests.post("http://localhost:5001/event/create", json=event_obj)
    assert response.json() == {'result': 'INVALID_JSON'}
    print("test_event_create()")
                                                                

def test_event_get():
    event_obj = {
        "name": faker_instance.company(),
        "location": faker_instance.address(),
        "start": faker_instance.date_time().strftime("%Y-%m-%d %H:%M:%S"),
        "duration": "some_days",
        "description": faker_instance.text(),
    }
    id = requests.post("http://localhost:5001/event/create", json=event_obj, headers={"X-User-Email":"admin@mail.com", "X-User-Level":"2"}).json()['id']
    response = requests.post("http://localhost:5001/event/get_one", json={"id": int(id)})    
    response = response.json()
    assert response['name'] == event_obj['name']
    assert response['location'] == event_obj['location']
    assert response['start'] == event_obj['start']
    assert response['description'] == event_obj['description']
    assert response['duration'] == event_obj['duration']
    print("test_event_get()")

def test_event_update():
    cur = con.cursor()
    cur.execute("DELETE FROM platform_events")
    con.commit()
    EVENTS_COUNT = 5
    events = list()
    for _ in range(EVENTS_COUNT):
        event_obj = {
            "name": faker_instance.company(),
            "location": faker_instance.address(),
            "start": faker_instance.date_time().strftime("%Y-%m-%d %H:%M:%S"),
            "duration": "some_days",
            "description": faker_instance.text(),
        }
        id = requests.post("http://localhost:5001/event/create", json=event_obj, headers={"X-User-Email":"admin@mail.com", "X-User-Level":"2"}).json()['id']
        event_obj["id"] = id
        events.append(event_obj)
    response = requests.post("http://localhost:5001/event/get_many", json={})
    response = response.json()
    assert len(response['events']) == EVENTS_COUNT
    sorted(response['events'], key=lambda x: x['id'])
    sorted(events, key=lambda x: x['id'])
    for i in range(EVENTS_COUNT):
        assert response['events'][i]['name'] == events[i]['name']
        assert response['events'][i]['location'] == events[i]['location']
        assert response['events'][i]['start'] == events[i]['start']
        assert response['events'][i]['description'] == events[i]['description']
        assert response['events'][i]['duration'] == events[i]['duration']
        assert int(response['events'][i]['id']) == int(events[i]['id'])
        
    print("test_event_update()")
    

test_event_create()
test_event_get()
test_event_update()