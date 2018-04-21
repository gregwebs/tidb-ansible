port module Main exposing (..)

-- Convert to javascript with `elm-make Main.elm --output=main.js`

import Http
import Array exposing (Array)

import Html exposing (..)
import Html.Attributes exposing (href, class, style)
import Html.Keyed as Keyed
import Html.Events
import Json.Decode
import Json.Decode as Decode
import Json.Decode exposing (int, string, float, bool, nullable, Decoder, map4, map3, list, field)
import Json.Encode as Encode

import Material
import Material.Button as Button
import Material.Card as Card
import Material.Color as Color
import Material.Options as Options exposing (css)
import Material.Toggles as Toggles
import Material.Textfield as Textfield
import Material.Progress
import Material.Layout as Layout
import Material.Elevation as Elevation
import Material.Grid exposing (align, grid, offset, cell, size, Device(..))
import Material.Scheme

-- type aliases for decoding messages from server
type alias RunningCommand =
  { command: List String
  , output: String
  , error: String
  , exit_code: Maybe Int
  }

type alias ServerState =
  { error : String
  , mysql_connect : String
  , command : Maybe RunningCommand
  }

default_server_state : ServerState
default_server_state =
  { error= ""
  , mysql_connect = ""
  , command = Nothing
  }

decodeRunningCommand : Decoder RunningCommand
decodeRunningCommand =
  map4 RunningCommand
    (field "command" <| list string)
    (field "output" string)
    (field "error" string)
    (field "exit_code" <| nullable int)

decodeServerState : Decoder ServerState
decodeServerState =
  map3 ServerState
    (field "error" string)
    (field "mysql_connect" string)
    (field "running" <| nullable decodeRunningCommand)

type ReadyState
  = Connecting
  | Open
  | Closed

type alias UIState = { selected_tab : Int }


type StepStatus
  = Next
  | Running
  | Idle
  | Completed

type alias Step =
    { name : String
    , recipe: String
    , status: StepStatus
    , description: String
    }

activeStep : Step -> Bool
activeStep step = step.status == Running || step.status == Next

type alias Model =
    { server_state : ServerState
    , steps : List Step
    , running_step : Maybe String
    , fail_msg : String
    , mdl : Material.Model
    , token : String
    , ready_state : ReadyState
    , ui_state : UIState
    }

init : ( Model, Cmd Msg )
init =
    ( { server_state = default_server_state
      , running_step = Nothing
      , steps =
        [ { name = "kube-up", recipe = "kube-resources", status = Next
        , description = """
Bring up a simulated cluster on Kubernetes via Minikube.
Builds an ansible docker image for the deploy steps.
Builds a centos docker image to simulate servers.
"""
          }
        , { name = "prepare", recipe = "ansible-prepare", status = Idle
          , description = "Run ansible prepare. This could take a while to download tidb binares."
          }
        , { name = "bootstrap", recipe = "ansible-bootstrap", status = Idle
        , description = """
Run ansible bootstrap. Sets up hosts for deployment.
"""
          }
        , { name = "deploy", recipe = "ansible-deploy", status = Idle
        , description = """
Run ansible deploy. Provisions machines with TiDB services.
"""
          }
        , { name = "start", recipe = "ansible-start", status = Idle
          , description = """
Run ansible start. Starts up TiDB services.
"""
          }
        ]
      , fail_msg = ""
      , mdl = Material.model
      , token = ""
      , ready_state = Connecting
      , ui_state = { selected_tab = 0 }
      }
    , Cmd.none )

type UIMsg
  = SelectTab Int

type Msg
  = NewServerState ServerState
  | NewToken String
  | InstallCommand String
  | Mdl (Material.Msg Msg)
  | FailedDecode String
  | NewReadyState ReadyState
  | CallbackDone (Result Http.Error String)
  | UIMsg UIMsg

active_step : Model -> Maybe Step
active_step model =
     List.filter activeStep model.steps
  |> List.head

update_steps : Model -> Model
update_steps model =
  case model.server_state.command of
    Nothing ->
      model

    Just rc ->
      let status = case rc.exit_code of
            Nothing ->
              Running

            Just 0 ->
              Completed

            Just _ ->
              -- TODO: failure state
              Next
      in
          case rc.command of
            "just" :: recipe :: [] ->
              if List.any (\step -> step.recipe == recipe) model.steps then
                  { model | fail_msg = ""
                          , running_step = case rc.exit_code of
                              Nothing -> Just recipe
                              Just _ -> Nothing
                          , steps = updateSteps Nothing status recipe model.steps }
              else
                  { model | fail_msg = String.append "did not find step: " recipe
                          , running_step = Nothing
                  }
            _ ->
              { model | running_step = Nothing
                      , fail_msg = String.append "unknown command: "
                <| String.join (" ") rc.command}

updateSteps : Maybe StepStatus -> StepStatus -> String -> List Step -> List Step
updateSteps nextStatus runningStatus recipe steps =
  case steps of
    [] ->
      []

    step::rest ->
      let isStep = recipe == step.recipe
          status = if isStep then runningStatus else
            case nextStatus of
              Nothing ->
                Completed
              Just status ->
                status
      in
         { step | status = status } ::
           updateSteps (if isStep
             then if runningStatus == Completed then Just Next else Just Idle
             else
                 case nextStatus of
                   Nothing ->
                     Nothing
                   Just _ ->
                     Just Idle
                 ) runningStatus recipe rest

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    NewServerState new_ss ->
      (update_steps {model | server_state = new_ss }, Cmd.none)

    NewToken token ->
        ({model | token = token}, Cmd.none)

    InstallCommand name ->
      (model, do_just_command model name)

    Mdl msg_ ->
        Material.update Mdl msg_ model

    FailedDecode str -> ({model | fail_msg = str}, Cmd.none)

    NewReadyState rs ->
        ({model | ready_state = rs}, Cmd.none)

    CallbackDone result ->
        (model, Cmd.none)

    UIMsg msg ->
        let (ui_state, cmd) = update_ui_state msg model
        in
            ({ model | ui_state = ui_state }, cmd)


-- | UIState does not generate commands
update_ui_state : UIMsg -> Model -> (UIState, Cmd Msg)
update_ui_state msg model =
  case msg of
    SelectTab num ->
        let cmd = case num of
          1 -> do_command model "mysql-connect"
          _ -> Cmd.none
        in
          let ui_state = model.ui_state
          in ({ ui_state | selected_tab = num }, cmd)


type alias Mdl =
    Material.Model

take_lines : Int -> String -> String
take_lines total str =
     String.split "\n" str
  |> List.reverse
  |> List.take total
  |> List.reverse
  |> String.join "\n"

view : Model -> Html Msg
view model =
  Material.Scheme.topWithScheme Color.Teal Color.LightGreen <|
    Layout.render Mdl model.mdl
    [ Layout.fixedHeader
    , Layout.selectedTab model.ui_state.selected_tab
    , Layout.onSelectTab (\msg -> UIMsg <| SelectTab msg)
    ]
    { header = [ h1 [ style [ ( "line-height", "1.2" ), ( "margin-left", "1rem") ] ] [ text "TiDB Installer" ] ]
    , drawer = []
    , tabs = ( [ text "Deploy Simulator"
               , text "View Cluster"
               ]
             , [ Color.background (Color.color Color.Teal Color.S400) ]
             )
    , main = [
        case model.ui_state.selected_tab of
          0 ->
            viewInstallTab model
          1 ->
            viewClusterTab model
          _ ->
            grid [] []
      ]
    }


viewClusterTab : Model -> Html Msg
viewClusterTab model =
  grid []
    [ cell [ size All 1 ] []
    , cell [ size All 10
      , Elevation.e6
      , css "padding-left" "30px"
      , css "padding-right" "30px"
      ]
      [ Card.view
          [ css "width" "100%"
          ]
          [ Card.title [] [ Card.head [] [text "MySQL connect"] ]
          , Card.text [ ] [ text model.server_state.mysql_connect ]
          ]
      ]
    ]


viewInstallTab : Model -> Html Msg
viewInstallTab model =
      grid []
        [ cell [ size All 1 ] []
        , cell [ size All 10
          , Elevation.e6
          , css "padding-left" "30px"
          , css "padding-right" "30px"
          ]
          [   h3 [] [text "Kubernetes Simulated Cluster Deployment"]
          ,   p [] (case active_step model of
                      Nothing ->   []
                      Just step ->
                        [ strong []
                          [ text <| String.join " "
                              ["Step", String.append step.name ": "]
                          ]
                        , text step.description
                        ]
                   )
          ,   div []
                  (model.steps
                    |> List.indexedMap (\i sq -> stepView model (i + 1) sq)
                  )
          , div [ style [ ("padding-left", "20px") ] ]
            (case model.running_step of
              Nothing ->
                [ Material.Progress.progress 0 ]
              Just _ ->
                [ Material.Progress.indeterminate
                ])
          , div [ style [("color", "red")] ] [
              text model.fail_msg
            , pre [] [ text <| defaultEmpty <| Maybe.map (\c -> c.error) model.server_state.command ]
            ]
          , Card.view
              [ css "width" "100%"
              ]
              [ Card.text [ css "overflow-x" "scroll" ]
                [ pre []
                  [ text <| take_lines 20
                         <| defaultEmpty
                         <| Maybe.map (\c -> c.output) model.server_state.command ] ]
              ]
          ]
        ]

defaultEmpty : Maybe String -> String
defaultEmpty = Maybe.withDefault ""

stepView : Model -> Int -> Step -> Html Msg
stepView model k step =
  let
    hue =
      Array.get ((k + 4) % Array.length Color.hues) Color.hues
        |> Maybe.withDefault Color.Teal

    shade =
      case step.status of
        Completed ->
          Color.S100

        _ ->
          Color.S500

    color =
      Color.color hue shade

  in
    Button.render Mdl [1] model.mdl
      [ Button.raised
      , Options.disabled <| not <| step.status == Next
      , Options.onClick (InstallCommand step.recipe)
      , css "width" "10em"
      , css "margin" "1em"
      , Color.background color
      , Color.text Color.primaryContrast
      , if activeStep step then Elevation.e8 else Elevation.e2
      ]
      [ text step.name ]


{-
isEnter : number -> Json.Decode.Decoder Msg
isEnter code =
   if code == 13 then
      Json.Decode.succeed SetNameOnServer
   else
      Json.Decode.fail "not Enter"
-}

do_command : Model -> String -> Cmd Msg
do_command model command =
  send_message model command (Encode.string "")

do_just_command : Model -> String -> Cmd Msg
do_just_command model value =
  send_message model "just" (Encode.string value)

callbackEncoded : String -> String -> Encode.Value -> Encode.Value
callbackEncoded token name args =
    let
        list =
            [ ( "token", Encode.string token )
            , ( "name", Encode.string name )
            , ( "args",  args )
            ]
    in
        list
            |> Encode.object

send_message : Model -> String -> Encode.Value -> Cmd Msg
send_message model name args =
    let
        body = Http.jsonBody (callbackEncoded model.token name args)
    in
        Http.send CallbackDone <|
            postCallback body

postCallback : Http.Body -> Http.Request String
postCallback body =
  Http.post "callback" body string

getServerStateOrFail : String -> Msg
getServerStateOrFail encoded =
  case Json.Decode.decodeString decodeServerState encoded of
    Ok (ssc) ->
      -- let _ = Debug.log "server state: " ssc in
      NewServerState ssc
    Err msg ->
      FailedDecode msg

decodeReadyState : Int -> Msg
decodeReadyState code =
  let _ = Debug.log "ready state: " code
  in
  case to_ready_state code of
    Ok(rs) ->
      NewReadyState rs
    Err msg ->
      FailedDecode msg

to_ready_state : Int -> Result String ReadyState
to_ready_state code =
  case code of
    0 -> Ok(Connecting)
    1 -> Ok(Open)
    2 -> Ok(Closed)
    _ -> Err("unknown ReadyState code")

port event_source_data : (String -> msg) -> Sub msg
port ready_state : (Int -> msg) -> Sub msg

subscriptions : Model -> Sub Msg
subscriptions model =
  Sub.batch
      [ Layout.subs Mdl model.mdl
      , event_source_data getServerStateOrFail
      , ready_state decodeReadyState
      ]

main : Program Never Model Msg
main =
    program { init = init, update = update, subscriptions = subscriptions, view = view }
